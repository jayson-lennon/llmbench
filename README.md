# Openrouter LLM Bencher

Directory structure:

```
llmbench/
├── prompts/
│   ├── category_A/
│   │   ├── bench_name_1/
│   │   │   ├── prompt_1.md
│   │   │   └── prompt_2.md
│   │   ├── bench_name_2/
│   │   │   └── prompt.md
│   │   └── ...
│   ├── category_B/
│   │   ├── another_bench/
│   │   │   ├── prompt.md
│   │   │   └── system.md
│   │   └── ...
│   └── ...
```

llmbench treats each `prompt*.md` file as an individual turn in the prompt. The prompt files will be alpha-sorted and then provided to the LLM in sequence, alternating between the prompt and the LLM in a chat format.

Bench directories containing a `system.md` file will use this as the system prompt. If no `system.md` is provided, then the value specified in `ssllmbr.toml` will be used instead.

