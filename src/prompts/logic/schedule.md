---
expected = "dr. chen"
---

You are scheduling a 1-hour team review meeting. Find the earliest time slot that all required attendees can join.

## Required Attendees
- **Alex** — free 09:00–12:00, 14:00–17:00
- **Beth**  — free 10:00–12:30, 13:00–16:00
- **Dr. Chen** — free 08:00–09:00, 11:00–15:00
- **Dana**  — free 09:00–12:00, 14:00–17:00

## Optional Attendees
- **Erik** — free 08:00–10:00, 13:00–17:00
- **Faye** — free 10:30–12:30, 14:00–16:30

## Rules
1. The meeting must be exactly 1 hour.
2. All four required attendees must be free for the entire slot.
3. Pick the earliest possible start time.

## Question
If you could remove one required attendee, the group could start earlier. Which required attendee is blocking the earliest possible start?

# **OUTPUT FORMAT**
A single line with the attendee's name (lowercase). No additional commentary.
