---
expected = "session data lost for all users"
---

You are a QA lead prioritizing bugs. Which bug should be fixed first?

## Priority Rules (apply in order)
1. **Data loss is highest priority.** Any bug that causes user data to be lost or corrupted is more urgent than functional failures.
2. **Among data loss bugs, prefer the one affecting all users.** A bug affecting all users takes precedence over one affecting only a subset.

## Bug Reports
- [ ] Login page fails to load for all users
- [ ] User data corrupted for Firefox users only
- [ ] Payment processing returns errors for all users
- [ ] Images fail to load on Safari only
- [ ] Session data lost for all users

# **OUTPUT FORMAT**
A single line with the bug to fix first. No additional commentary.
