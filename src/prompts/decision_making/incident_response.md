---
expected = "verify the most recent backup"
---

You are an incident responder. An alert has fired. Which action should be taken FIRST?

## Incident Response Rules (apply in order)
1. **Production database: verify backup first.** If the affected system is a production database, you MUST verify the most recent backup before taking any other action.
2. **Active exfiltration: isolate the network.** If data is actively being exfiltrated, isolate the network immediately.
3. **PII involved: notify compliance.** If personally identifiable information is stored in the affected system, notify the compliance team.

## Current Incident
An active network connection from an unknown IP address is detected to the production customer database. This database stores customer names, emails, and payment information. Data is currently being transferred outbound.

## Available Actions
- [ ] Verify the most recent backup
- [ ] Isolate the network
- [ ] Notify the compliance team
- [ ] Restart the affected server
- [ ] Rotate all access credentials

# **OUTPUT FORMAT**
A single line with the action to take first. No additional commentary.
