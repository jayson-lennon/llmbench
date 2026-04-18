---
expected = "security"
---

You are a support ticket routing agent. Assign each ticket to the **first matching category** using the rules below. Rules are checked in priority order — the highest-priority match wins, even if lower-priority rules also match.

## Categories (highest priority first)

1. **SECURITY** — Login from an unrecognized IP **and** a password change within the last 60 minutes.
2. **BILLING** — Invoice amount differs from the agreed rate by **more than** 10%.
3. **OUTAGE** — More than 3 users report the same error message within 15 minutes.
4. **ONBOARDING** — Account created within the last 7 days **and** this is the user's first support message.
5. **GENERAL** — All other tickets.

## Ticket

- **User account created:** 2 days ago
- **Previous support messages:** None (this is the first)
- **Last login IP:** 203.0.113.42 (not in recognized IP list)
- **Password last changed:** 45 minutes ago
- **Latest invoice:** $560
- **Agreed monthly rate:** $500
- **Other users with same error in last 15 min:** 1

## Question

Which category should this ticket be assigned to?

# **OUTPUT FORMAT**

A single line with the category name (lowercase). No additional commentary.
Example: `billing`
