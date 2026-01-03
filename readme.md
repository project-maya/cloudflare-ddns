# Cloudflare Dynamic DNS Updater

A Rust-based dynamic DNS (DDNS) client that automatically updates your Cloudflare DNS records with your current public IP address. Supports both IPv4 (A records) and IPv6 (AAAA records).

[![GitHub release](https://img.shields.io/github/release/project-maya/cloudflare-ddns.svg)](https://github.com/YOUR_USERNAME/cloudflare-ddns/releases)
[![Build Status](https://github.com/YOUR_USERNAME/cloudflare-ddns/workflows/Release/badge.svg)](https://github.com/project-maya/cloudflare-ddns/actions)

## Features

- 🔄 Automatic IP detection (IPv4 and IPv6)
- 🔍 Compares current IP with existing DNS records
- ✏️ Updates records only when IP changes
- ➕ Creates new records if they don't exist
- 📝 YAML-based configuration
- 🛡️ Secure API token authentication
- 🚀 Fast and lightweight
- ⚡ Asynchronous operations

## Prerequisites

- Rust 1.70 or higher
- A Cloudflare account with a domain
- Cloudflare API token with DNS edit permissions

## Installation

### From Source

1. Clone the repository:
```bash
git clone https://github.com/yourusername/cloudflare-ddns.git
cd cloudflare-ddns
```
2. Build the project:
```bash
cargo build --release
```
3. The binary will be available at `target/release/cloudflare-ddns`

## Configuration

### 1. Create a Cloudflare API Token
1. Log in to your Cloudflare Dashboard
2. Go to My Profile → API Tokens
3. Click Create Token
4. Use the Edit zone DNS template or create a custom token with:
  - Permissions: Zone → DNS → Edit
  - Zone Resources: Include → Specific zone → Select your domain
5. Copy the generated token

### 2. Get Your Zone ID
1. In the Cloudflare Dashboard, select your domain
2. Scroll down on the Overview page
3. Find Zone ID in the right sidebar and copy it

### 3. Configure the Application
Create a config.yml file in the same directory as the executable:
```yaml
cloudflare:
  api_token: "YOUR_API_TOKEN_HERE"
  zone_id: "YOUR_ZONE_ID_HERE"
  
records:
  - name: "example.com"
    type: "A"
  - name: "subdomain.example.com"
    type: "A"
  - name: "example.com"
    type: "AAAA"
```
### Configuration Options
- api_token: Your Cloudflare API token (required)
- zone_id: Your Cloudflare zone ID (required)
- records: List of DNS records to manage
  - name: Fully qualified domain name (e.g., example.com or subdomain.example.com)
  - type: Record type (A for IPv4 or AAAA for IPv6)

## Usage

### Run Once
```bash
./cloudflare-ddns
```
Or with cargo:
```bash
cargo run --release
```

## Example Output
```
Processing A record for example.com
Current public IP (A): 203.0.113.42
IP mismatch! Updating record from 203.0.113.10 to 203.0.113.42
✓ Record updated successfully

Processing AAAA record for example.com
Current public IP (AAAA): 2001:db8::1
✓ Record already up to date

✓ All records processed successfully!
```

## Automation

### Linux/macOS (systemd)
Create a systemd service file at `/etc/systemd/system/cloudflare-ddns.service`:
```ini
[Unit]
Description=Cloudflare Dynamic DNS Updater
After=network-online.target
Wants=network-online.target

[Service]
Type=oneshot
ExecStart=/path/to/cloudflare-ddns
WorkingDirectory=/path/to/config/directory
User=yourusername

[Install]
WantedBy=multi-user.target
```
Create a timer at `/etc/systemd/system/cloudflare-ddns.timer`:
```ini
[Unit]
Description=Run Cloudflare DDNS every 5 minutes

[Timer]
OnBootSec=1min
OnUnitActiveSec=5min
Persistent=true

[Install]
WantedBy=timers.target
```
Enable and start:
```bash
sudo systemctl daemon-reload
sudo systemctl enable cloudflare-ddns.timer
sudo systemctl start cloudflare-ddns.timer
```
### Linux/macOS (cron)
Add to crontab `(crontab -e)`:
```bash
*/5 * * * * /path/to/cloudflare-ddns >> /var/log/cloudflare-ddns.log 2>&1
```
### Windows (Task Scheduler)
1. Open Task Scheduler
2. Create Basic Task
3. Set trigger (e.g., every 5 minutes)
4. Action: Start a program → Browse to cloudflare-ddns.exe
5. Set "Start in" to the directory containing config.yml

## Troubleshooting
### "Failed to read config.yml"
Ensure `config.yml` is in the same directory as the executable or specify the working directory.

### "API errors: ..."
- Verify your API token has the correct permissions
- Check that your zone ID is correct
- Ensure your API token hasn't expired

### "Failed to get public IP"
- Check your internet connection
- Verify you can access `https://api.ipify.org` (for IPv4) and `https://api64.ipify.org` (for IPv6)
- If behind a firewall, ensure outbound HTTPS is allowed

### IPv6 not working
- Ensure your network supports IPv6
- Some networks don't provide IPv6 connectivity
- The program will fail gracefully if IPv6 is unavailable

### Dependencies
- `reqwest` - HTTP client
- `serde` - Serialization framework
- `serde_yaml` - YAML parsing
- `tokio` - Async runtime
- `anyhow` - Error handling

## Security Best Practices
1. Use API tokens, not API keys - Tokens have limited scope
2. Restrict token permissions - Only grant DNS edit access
3. Rotate tokens periodically - Update tokens every few months
4. Store config securely - Set appropriate file permissions:
```bash
chmod 600 config.yml
```

## Contributing
Contributions are welcome! Please feel free to submit a Pull Request.
1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License
This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments
- Cloudflare API Documentation
- ipify API for public IP detection

## Support
If you encounter any issues or have questions:
- Open an issue on GitHub
- Check existing issues for solutions
- Review the Cloudflare API documentation

## Changelog
v0.1.0 
- Initial release
- Support for A and AAAA records
- Automatic IP detection
- Record creation and updates
- YAML configuration
---
Note: This tool is not affiliated with Cloudflare. Use at your own risk.
