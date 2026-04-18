---
expected = "block"
---

You are a release engineer. Should this deployment proceed or be blocked?

## Deployment Gate Rules

All three conditions must be satisfied to proceed. If ANY condition fails, the deployment is blocked.

1. **All CI tests must pass.** Even a single failing test blocks the deployment.
2. **No critical security vulnerabilities.** The security scan must report zero critical findings.
3. **Rollback plan must exist.** A tested rollback procedure must be documented.

## Current Deployment Status

- CI Tests: 841 passed, 1 failed
- Security Scan: 3 low, 2 medium, 0 critical
- Rollback Plan: Documented and tested

# **OUTPUT FORMAT**

A single line: either "proceed" or "block". No additional commentary.
Example: `block`
