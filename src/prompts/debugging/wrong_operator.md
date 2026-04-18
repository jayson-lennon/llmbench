---
expected = "no discount"
---

You are reviewing a colleague's code. A store gives discounts to customers who are members **or** whose order totals more than $100. The code below has a bug — it uses the wrong logical operator.

What does the program print when run?

```python
def check_discount(is_member, order_total):
    if is_member and order_total > 100:  # BUG: should use 'or'
        return "discount"
    else:
        return "no discount"

print(check_discount(False, 150))
```

**Important:** Trace the code exactly as written. Do not fix bugs.

# **OUTPUT FORMAT**

A single line with the exact output the program prints. No additional commentary.
Example: `discount`
