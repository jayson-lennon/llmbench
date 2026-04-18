---
expected = "jal 405"
---

You are a travel agent. A client needs to fly from Helsinki (HEL) to Tokyo (NRT), arriving no later than 14:00 local time on a weekday. Which flight should you book?

## Available Flights (all departures are same-day, times are local)

| Flight     | Origin | Destination | Departs | Arrives | Days           | Stops |
|------------|--------|-------------|---------|---------|----------------|-------|
| AY 71      | HEL    | NRT         | 17:20   | 09:25+1 | Daily          | 0     |
| SK 1045    | HEL    | ARN         | 06:45   | 07:00   | Mon–Fri        | —     |
| SK 405     | ARN    | NRT         | 09:00   | 06:30+1 | Mon–Fri        | 0     |
| JL 405     | HEL    | NRT         | 10:30   | 05:45+1 | Mon, Wed, Fri  | 1     |
| LH 307     | HEL    | FRA         | 08:15   | 10:00   | Daily          | —     |
| NH 4793    | FRA    | NRT         | 13:20   | 08:30+1 | Tue, Thu, Sat  | 0     |
| BA 797     | HEL    | LHR         | 07:30   | 09:05   | Daily          | —     |
| BA 5       | LHR    | NRT         | 11:00   | 07:45+1 | Daily          | 0     |

`+1` = arrives the next day (Japan is UTC+9, Finland is UTC+2).

## Selection Rules
1. Arrival must be by 14:00 local time.
2. Fewer stops is better.
3. Among equal options, prefer the earlier arrival.

# **OUTPUT FORMAT**
A single line with the flight code (e.g. "AY 71"). Use lowercase. No additional commentary.
