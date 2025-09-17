# Docker Image Attestation Implementation

## Problem Statement

Currently, HOPR Docker images (`hoprd`, `hopli`, `hopr-pluto`) are built and published to the Google Container Registry without cryptographic attestation. This creates security vulnerabilities and prevents users from verifying the authenticity and integrity of the images they are deploying.

## Description

Implement comprehensive Docker image attestation for all HOPR Docker images to ensure supply chain security and enable users to verify image authenticity. This should include:

1. **Image Signing**: Cryptographically sign Docker images during the CI/CD pipeline
2. **Attestation Generation**: Generate and attach attestations containing build metadata, SBOM, and provenance information
3. **Verification Documentation**: Provide clear instructions for users to verify image signatures and attestations
4. **Integration with Existing Infrastructure**: Leverage existing GPG signing capabilities and extend the current Nix-based Docker build process

## Current State Analysis

### Existing Infrastructure
- Docker images are built using Nix (`nix/docker-builder.nix`)
- Images are published via GitHub Actions (`.github/workflows/build-docker.yaml`)
- Registry: `europe-west3-docker.pkg.dev/hoprassociation/docker-images/`
- Current images: `hoprd`, `hopli`, `hopr-pluto` (with variants: dev, profile)
- GPG signing capability already exists in `justfile` for file signing

### Current Vulnerabilities
- No cryptographic verification of image authenticity
- No build provenance tracking
- No software bill of materials (SBOM) generation
- Users cannot verify if images are tampered with
- Supply chain attacks are possible

## Proposed Implementation

### Phase 1: Image Signing with Cosign
1. **Integrate Cosign** into the Docker build pipeline
2. **Keyless signing** using GitHub OIDC tokens for CI/CD builds
3. **Key-based signing** for release builds using existing GPG infrastructure
4. **Automatic signature** generation for all published images

### Phase 2: Attestation Generation
1. **SLSA Build Provenance**: Generate Level 2+ SLSA provenance attestations
2. **SBOM Generation**: Create software bill of materials for each image
3. **Build Metadata**: Include commit SHA, build timestamp, build environment details
4. **Vulnerability Scanning**: Integrate image vulnerability scanning results

### Phase 3: Verification and Documentation
1. **Verification Scripts**: Provide scripts for image verification
2. **User Documentation**: Update README with verification instructions
3. **CI/CD Integration**: Add verification steps to deployment workflows
4. **Monitoring**: Set up monitoring for unsigned/unverified images

## Technical Requirements

### Tools and Dependencies
- **Cosign**: For image signing and verification
- **SLSA-GitHub-Generator**: For provenance attestation
- **Syft** or **Grype**: For SBOM generation
- **GitHub OIDC**: For keyless signing in CI/CD
- Integration with existing **Nix build system**

### Security Considerations
- Secure key management for release signing
- Rotation policy for signing keys
- Audit logging for signing operations
- Multi-signature support for critical releases
- Integration with existing GPG key infrastructure

### Infrastructure Changes
1. **Extend `nix/docker-builder.nix`** to support attestation metadata
2. **Update `.github/workflows/build-docker.yaml`** to include signing steps
3. **Add verification steps** to deployment workflows
4. **Create new justfile targets** for signing and verification operations

## Acceptance Criteria

### Must Have
- [ ] All published Docker images are cryptographically signed
- [ ] SLSA Build Provenance attestations are generated and attached
- [ ] Users can verify image signatures using documented procedures
- [ ] CI/CD pipeline automatically signs images without manual intervention
- [ ] Verification fails for tampered or unsigned images

### Should Have
- [ ] SBOM generation for all images
- [ ] Vulnerability attestations attached to images
- [ ] Multi-signature support for production releases
- [ ] Automated verification in deployment workflows
- [ ] Integration with existing GPG key management

### Could Have
- [ ] Historical signature verification for existing images
- [ ] Integration with admission controllers for Kubernetes deployments
- [ ] Signature transparency log integration
- [ ] Custom policy definitions for image verification

## Implementation Plan

### 1. Research and Design (1-2 weeks)
- Evaluate Cosign vs alternatives (Notary v2, Docker Content Trust)
- Design key management strategy
- Plan integration with existing Nix build system
- Define attestation schema and metadata requirements

### 2. Core Implementation (2-3 weeks)
- Integrate Cosign into Docker build pipeline
- Implement keyless signing for CI builds
- Add signing steps to GitHub Actions workflow
- Create basic verification documentation

### 3. Enhanced Attestation (2-3 weeks)
- Implement SLSA provenance generation
- Add SBOM generation to build process
- Integrate vulnerability scanning results
- Enhance verification scripts and documentation

### 4. Testing and Documentation (1-2 weeks)
- Comprehensive testing of signing and verification
- Update user documentation and examples
- Create troubleshooting guides
- Performance testing of build pipeline changes

### 5. Deployment and Monitoring (1 week)
- Gradual rollout to different image variants
- Set up monitoring and alerting
- Train team on new processes
- Post-deployment verification and fixes

## Security Benefits

1. **Supply Chain Security**: Prevents malicious image substitution
2. **Integrity Verification**: Users can verify images haven't been tampered with
3. **Provenance Tracking**: Full visibility into build process and dependencies
4. **Compliance**: Meets security requirements for enterprise deployments
5. **Trust Building**: Increases user confidence in HOPR infrastructure

## User Experience Impact

### For End Users
- Simple verification commands: `cosign verify [image]`
- Clear documentation on verification process
- Automated verification in example deployments
- Enhanced security without complexity

### For Developers
- Transparent integration with existing workflows
- Additional metadata for debugging and auditing
- Enhanced security posture for the project
- Industry-standard security practices

## Related Work and Standards

- **SLSA Framework**: https://slsa.dev/
- **Cosign**: https://docs.sigstore.dev/cosign/overview/
- **NIST SSDF**: Software supply chain security framework
- **CNCF Security Best Practices**: Container image security guidelines

## Risk Assessment

### Low Risk
- Integration complexity with Nix build system
- Minor performance impact on build times
- Learning curve for team members

### Medium Risk
- Key management and rotation procedures
- Potential CI/CD pipeline failures during rollout
- Backwards compatibility with existing tooling

### Mitigation Strategies
- Phased rollout with fallback options
- Comprehensive testing in staging environment
- Clear rollback procedures and monitoring
- Team training and documentation

## Success Metrics

1. **100% of published images** are signed and verified
2. **Zero security incidents** related to image tampering
3. **Documentation compliance** - all users can successfully verify images
4. **Build pipeline reliability** - <5% increase in build failure rate
5. **Adoption metrics** - tracking verification usage in the community

---

**Labels**: `security`, `docker`, `infrastructure`, `epic`
**Priority**: High
**Estimated Effort**: 6-10 weeks
**Dependencies**: Access to signing keys, CI/CD pipeline permissions