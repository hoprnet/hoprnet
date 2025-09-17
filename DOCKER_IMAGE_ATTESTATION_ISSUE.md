---
name: Docker Image Attestation Implementation
about: Implement cryptographic attestation for Docker images to enhance supply chain security
title: "Implement Docker Image Attestation for Supply Chain Security"
labels: "security, docker, infrastructure, epic, status:new"
assignees: ""
---

## Problem Statement

Currently, HOPR Docker images (`hoprd`, `hopli`, `hopr-pluto`) are built and published to the Google Container Registry without cryptographic attestation. This creates security vulnerabilities and prevents users from verifying the authenticity and integrity of the images they are deploying.

## Description

Implement comprehensive Docker image attestation for all HOPR Docker images to ensure supply chain security and enable users to verify image authenticity. This addresses a critical gap in our current security posture.

### Current State Analysis

**Existing Infrastructure:**
- Docker images built using Nix (`nix/docker-builder.nix`)
- Images published via GitHub Actions (`.github/workflows/build-docker.yaml`) 
- Registry: `europe-west3-docker.pkg.dev/hoprassociation/docker-images/`
- Current images: `hoprd`, `hopli`, `hopr-pluto` (with dev/profile variants)
- GPG signing capability exists in `justfile` for file signing

**Current Vulnerabilities:**
- ❌ No cryptographic verification of image authenticity
- ❌ No build provenance tracking  
- ❌ No software bill of materials (SBOM) generation
- ❌ Users cannot verify if images are tampered with
- ❌ Supply chain attacks are possible

## Proposed Solution

### Phase 1: Image Signing with Cosign
- Integrate Cosign into Docker build pipeline
- Implement keyless signing using GitHub OIDC tokens for CI/CD builds
- Use key-based signing for release builds leveraging existing GPG infrastructure
- Automatic signature generation for all published images

### Phase 2: Attestation Generation  
- Generate SLSA Build Provenance attestations (Level 2+)
- Create software bill of materials (SBOM) for each image
- Include build metadata (commit SHA, timestamp, environment)
- Integrate vulnerability scanning results

### Phase 3: Verification and Documentation
- Provide user-friendly verification scripts
- Update README with clear verification instructions
- Add verification steps to deployment workflows
- Set up monitoring for unsigned/unverified images

## Technical Implementation

### Required Tools
- **Cosign**: Image signing and verification
- **SLSA-GitHub-Generator**: Provenance attestation
- **Syft/Grype**: SBOM generation
- **GitHub OIDC**: Keyless signing
- Integration with existing **Nix build system**

### Infrastructure Changes
1. Extend `nix/docker-builder.nix` to support attestation metadata
2. Update `.github/workflows/build-docker.yaml` with signing steps
3. Add verification steps to deployment workflows  
4. Create new justfile targets for signing/verification operations

## Acceptance Criteria

### Must Have ✅
- [ ] All published Docker images are cryptographically signed
- [ ] SLSA Build Provenance attestations generated and attached
- [ ] Users can verify image signatures using documented procedures
- [ ] CI/CD pipeline automatically signs images without manual intervention
- [ ] Verification fails for tampered or unsigned images

### Should Have 🎯
- [ ] SBOM generation for all images
- [ ] Vulnerability attestations attached to images
- [ ] Multi-signature support for production releases
- [ ] Automated verification in deployment workflows
- [ ] Integration with existing GPG key management

### Could Have 💡
- [ ] Historical signature verification for existing images
- [ ] Integration with Kubernetes admission controllers
- [ ] Signature transparency log integration
- [ ] Custom policy definitions for image verification

## Security Benefits

1. **Supply Chain Security**: Prevents malicious image substitution
2. **Integrity Verification**: Users can verify images haven't been tampered with  
3. **Provenance Tracking**: Full visibility into build process and dependencies
4. **Compliance**: Meets enterprise security requirements
5. **Trust Building**: Increases user confidence in HOPR infrastructure

## User Experience Impact

### For End Users
- Simple verification: `cosign verify [image]`
- Clear documentation and examples
- Automated verification in deployments
- Enhanced security without added complexity

### For Developers  
- Transparent integration with existing workflows
- Rich metadata for debugging and auditing
- Industry-standard security practices
- Enhanced project security posture

## Implementation Timeline

| Phase | Duration | Key Deliverables |
|-------|----------|------------------|
| Research & Design | 1-2 weeks | Tool evaluation, architecture design |
| Core Implementation | 2-3 weeks | Cosign integration, basic signing |
| Enhanced Attestation | 2-3 weeks | SLSA provenance, SBOM generation |
| Testing & Documentation | 1-2 weeks | User guides, verification testing |
| Deployment & Monitoring | 1 week | Rollout, monitoring, team training |

**Total Estimated Effort**: 6-10 weeks

## Success Metrics

- ✅ **100% of published images** are signed and verified
- ✅ **Zero security incidents** related to image tampering  
- ✅ **Documentation compliance** - all users can verify images
- ✅ **Build pipeline reliability** - <5% increase in build failure rate
- ✅ **Community adoption** - tracking verification usage

## Related Standards and References

- [SLSA Framework](https://slsa.dev/) - Supply chain security framework
- [Cosign Documentation](https://docs.sigstore.dev/cosign/overview/) - Image signing tool
- [NIST SSDF](https://csrc.nist.gov/Projects/ssdf) - Software supply chain security
- [CNCF Security Best Practices](https://github.com/cncf/tag-security/blob/main/security-whitepaper/v1/CNCF_cloud-native-security-whitepaper-Nov2020.pdf) - Container security guidelines

## Additional Context

This implementation will leverage our existing security infrastructure (GPG keys, CI/CD pipelines) while adding industry-standard image attestation capabilities. The phased approach ensures minimal disruption to current workflows while providing immediate security benefits.

The solution aligns with CNCF security best practices and prepares HOPR for enterprise adoption by meeting supply chain security requirements that are increasingly becoming mandatory in production environments.