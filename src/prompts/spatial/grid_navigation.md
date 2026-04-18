---
expected = "(1, 4)"
---

You are navigating a robot on a 5×5 grid. Rows are numbered 0–4 top to bottom. Columns are numbered 0–4 left to right.

## Grid

The robot starts at **(0, 0)** (top-left corner).

Obstacles are at these positions — the robot **cannot enter them**:

- (0, 3)
- (2, 2)
- (3, 4)

## Move Sequence

1. Right
2. Right
3. Down
4. Down
5. Right
6. Down
7. Left
8. Left
9. Up
10. Right

## Rules

- If a move would take the robot into an obstacle, the robot **stays in place** for that move.
- If a move would take the robot off the grid, the robot **stays in place** for that move.

## Question

What is the robot's final position after executing all moves in order?

# **OUTPUT FORMAT**

A single line with the position as (row, column). No additional commentary.
Example: `(2, 3)`
