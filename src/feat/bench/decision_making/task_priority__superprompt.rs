use crate::feat::bench::prelude::*;

impl_simple_bench!(
    "decision_making/task_priority__superprompt",
    r#"
You are an expert game developer and project manager. Your job is to determine the task that should be completed next given a list of unordered tasks.

There is only 1 developer working on the game, so the task should provide the most value in driving the project forward. The most important task in a game development project is one that is blocking other incomplete tasks.

## Building Graphics
- [x] Residential zone
- [ ] Commercial zone
- [x] Industrial zone
- [ ] Utilities
- [x] Emergency Response

## Sound Effects
- [ ] Ambient city noise
- [ ] UI

## Technical Implementation
- [ ] Population monitor
- [ ] Loan management
- [ ] Pick a game engine
- [ ] Zone grid placement
- [ ] Calculating revenue based on zone capacity and usage levels

## Game Design
- [ ] Loss conditions
  - [x] Out of money
  - [ ] City destroyed
  - [ ] Unhappy residents
- [ ] Win conditions
- [ ] Keyboard shortcuts
- [x] Advisors
- [x] Road building algorithm
- [x] Weather transitions
- [ ] Available zones and buildings

## Animations
- [ ] People walking
- [ ] Cars driving
- [ ] Title screen

# **OUTPUT FORMAT**
A single line with the task to work on. No additional commentary.
    "#,
    expect_response!("pick a game engine")
);
