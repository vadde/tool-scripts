# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| latest  | :white_check_mark: |

## Reporting a Vulnerability

We take security seriously. If you discover a security vulnerability within any
tool in this repository, please report it responsibly.

### How to Report

1. **DO NOT** open a public GitHub issue for security vulnerabilities
2. Email your findings to **[INSERT SECURITY EMAIL]**
3. Include the following in your report:
   - Description of the vulnerability
   - Steps to reproduce
   - Affected tool(s) and version(s)
   - Potential impact assessment
   - Suggested fix (if any)

### What to Expect

- **Acknowledgment**: Within 48 hours of your report
- **Assessment**: Within 5 business days, we will assess the vulnerability
- **Resolution**: We aim to resolve critical vulnerabilities within 14 days
- **Disclosure**: We will coordinate public disclosure with you

### Scope

This security policy covers all tools and scripts within this repository,
including:

- Source code in `tools/`
- Automation scripts in `scripts/`
- CI/CD workflows in `.github/workflows/`
- Dependencies declared in any manifest files

### Out of Scope

- Vulnerabilities in upstream dependencies (report to the upstream project)
- Social engineering attacks
- Denial of service attacks against GitHub infrastructure

## Security Best Practices for Contributors

When contributing to this repository:

1. **Never commit secrets** — Use environment variables or secret managers
2. **Pin dependencies** — Use exact versions, not ranges
3. **Validate inputs** — All tools must validate and sanitize user inputs
4. **Principle of least privilege** — Request only necessary permissions
5. **Review CI/CD changes** — Workflow changes require maintainer review

## Security Scanning

This repository employs automated security scanning:

- **Dependency scanning** — Weekly via GitHub Actions
- **Secret detection** — Pre-commit hooks and CI checks
- **Static analysis** — Language-specific SAST tools in CI pipeline
