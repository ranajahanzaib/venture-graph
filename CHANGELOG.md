# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Source-aware co-investment discovery and recent-observation rankings
- Public project roadmap and contribution guidance

### Changed

- Local Compose ports bind to loopback; the API container runs as a non-root user
- Environment files and imported datasets are excluded from Git and Docker build contexts
- Documentation now distinguishes current capabilities, limitations, and source evidence

### Security

- CI uses read-only repository permissions
- Security vulnerability reports use GitHub's private advisory workflow

### Documentation

- Conduct reports and security vulnerability reports now use separate contact routes
