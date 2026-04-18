---
expected = "[5, 3, 1]"
---

What does the following Python program print?

```python
items = [1, 2, 3, 4, 5]
result = []
i = 0
while i < len(items):
    if items[i] % 2 == 1:
        result.append(items[i])
    i += 1
result.reverse()
print(result)
```

# **OUTPUT FORMAT**
A single line with the exact output including brackets. No additional commentary.
