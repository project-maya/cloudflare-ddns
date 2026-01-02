use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Deserialize)]
struct Config {
    cloudflare: CloudflareConfig,
    records: Vec<RecordConfig>,
}

#[derive(Debug, Deserialize)]
struct CloudflareConfig {
    api_token: String,
    zone_id: String,
}

#[derive(Debug, Deserialize)]
struct RecordConfig {
    name: String,
    #[serde(rename = "type")]
    record_type: String,
}

#[derive(Debug, Deserialize)]
struct CloudflareResponse {
    success: bool,
    result: Option<Vec<DnsRecord>>,
    errors: Option<Vec<ApiError>>,
}

#[derive(Debug, Deserialize)]
struct SingleRecordResponse {
    success: bool,
    result: Option<DnsRecord>,
    errors: Option<Vec<ApiError>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct DnsRecord {
    id: Option<String>,
    name: String,
    #[serde(rename = "type")]
    record_type: String,
    content: String,
    ttl: Option<u32>,
    proxied: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct ApiError {
    code: u32,
    message: String,
}

// Remove the IpResponse struct since we're getting plain text
// We'll use a simple string for IP responses

struct CloudflareClient {
    client: reqwest::Client,
    api_token: String,
    zone_id: String,
}

impl CloudflareClient {
    fn new(api_token: String, zone_id: String) -> Self {
        let client = reqwest::Client::new();
        Self {
            client,
            api_token,
            zone_id,
        }
    }

    async fn get_dns_records(&self, name: &str, record_type: &str) -> Result<Vec<DnsRecord>> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
            self.zone_id
        );

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .query(&[("name", name), ("type", record_type)])
            .send()
            .await?;

        let cf_response: CloudflareResponse = response.json().await?;

        if !cf_response.success {
            if let Some(errors) = cf_response.errors {
                anyhow::bail!("API errors: {:?}", errors);
            }
        }

        Ok(cf_response.result.unwrap_or_default())
    }

    async fn create_dns_record(&self, name: &str, record_type: &str, content: &str) -> Result<DnsRecord> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records",
            self.zone_id
        );

        let record = DnsRecord {
            id: None,
            name: name.to_string(),
            record_type: record_type.to_string(),
            content: content.to_string(),
            ttl: Some(1), // Auto TTL
            proxied: Some(false),
        };

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .json(&record)
            .send()
            .await?;

        let cf_response: SingleRecordResponse = response.json().await?;

        if !cf_response.success {
            if let Some(errors) = cf_response.errors {
                anyhow::bail!("API errors: {:?}", errors);
            }
        }

        cf_response.result.context("No result returned")
    }

    async fn update_dns_record(&self, record_id: &str, name: &str, record_type: &str, content: &str) -> Result<DnsRecord> {
        let url = format!(
            "https://api.cloudflare.com/client/v4/zones/{}/dns_records/{}",
            self.zone_id, record_id
        );

        let record = DnsRecord {
            id: None,
            name: name.to_string(),
            record_type: record_type.to_string(),
            content: content.to_string(),
            ttl: Some(1),
            proxied: Some(false),
        };

        let response = self
            .client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .json(&record)
            .send()
            .await?;

        let cf_response: SingleRecordResponse = response.json().await?;

        if !cf_response.success {
            if let Some(errors) = cf_response.errors {
                anyhow::bail!("API errors: {:?}", errors);
            }
        }

        cf_response.result.context("No result returned")
    }
}

async fn get_public_ip(ip_type: &str) -> Result<String> {
    let client = reqwest::Client::new();
    
    let url = match ip_type {
        "A" => "https://ipinfo.io/ip",
        "AAAA" => "https://ifconfig.me/ip",
        _ => anyhow::bail!("Unsupported record type: {}", ip_type),
    };

    let response = client.get(url).send().await?;
    let ip = response.text().await?;
    
    // Remove any whitespace or newlines
    let ip = ip.trim().to_string();
    
    // Validate IP format
    if ip.is_empty() {
        anyhow::bail!("Empty IP response");
    }
    
    Ok(ip)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config_content = fs::read_to_string("config.yml")
        .context("Failed to read config.yml")?;
    let config: Config = serde_yaml::from_str(&config_content)
        .context("Failed to parse config.yml")?;

    let client = CloudflareClient::new(
        config.cloudflare.api_token,
        config.cloudflare.zone_id,
    );

    // Process each record
    for record_config in config.records {
        println!("\nProcessing {} record for {}", record_config.record_type, record_config.name);

        // Get current public IP
        let current_ip = get_public_ip(&record_config.record_type).await?;
        println!("Current public IP ({}): {}", record_config.record_type, current_ip);

        // Get existing DNS records
        let existing_records = client
            .get_dns_records(&record_config.name, &record_config.record_type)
            .await?;

        if let Some(existing) = existing_records.first() {
            // Record exists - check if update needed
            if existing.content != current_ip {
                println!("IP mismatch! Updating record from {} to {}", existing.content, current_ip);
                client
                    .update_dns_record(
                        existing.id.as_ref().unwrap(),
                        &record_config.name,
                        &record_config.record_type,
                        &current_ip,
                    )
                    .await?;
                println!("✓ Record updated successfully");
            } else {
                println!("✓ Record already up to date");
            }
        } else {
            // Record doesn't exist - create it
            println!("Record not found. Creating new record...");
            client
                .create_dns_record(&record_config.name, &record_config.record_type, &current_ip)
                .await?;
            println!("✓ Record created successfully");
        }
    }

    println!("\n✓ All records processed successfully!");
    Ok(())
}