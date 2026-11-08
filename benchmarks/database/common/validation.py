"""Correctness checks and parameter formulas shared by the harness.

Every case in workloads/cases.json carries SQL checks with expected values. The
expectation strings are tiny expressions over the run parameters, so the values
come from the dataset definition rather than from any client's output.

The checks are evaluated twice:
  1. by the client, through its own driver, and reported in its JSON;
  2. by the harness, through psql, after the client exits.
A case only passes when both agree with the expectation.

The parameter formulas live here for the harness (it needs them for EXPLAIN
literals) and are duplicated in every client; workloads/README.md is the single
description of what they mean.
"""

STATUSES = ["new", "paid", "shipped", "cancelled"]
CATEGORIES = ["books", "games", "tools", "music"]
CITIES = ["Bengaluru", "Berlin", "Lisbon", "Osaka"]


def gen(name, i, rows, worker=0):
    if name == "id":
        return i + 1
    if name == "k":
        return (i * 37) % rows
    if name == "k_hi":
        return ((i * 37) % rows) + 25
    if name == "pick_id":
        return ((i * 7) % rows) + 1
    if name == "pick_product":
        return ((i * 7) % 1000) + 1
    if name == "offset":
        return (i * 100) % 1000
    if name == "qty":
        return (i % 5) + 1
    if name == "order_id":
        return ((i * 997) % 100000) + 1
    if name in ("app_product_id", "app_customer_id"):
        return 100000 + i
    if name == "app_order_id":
        return 1000000 + i
    if name == "worker_id":
        return worker * 1000000 + i + 1
    if name == "payload":
        return "payload-" + str(i)
    if name == "payload2":
        return "updated-" + str(i)
    if name == "tag":
        return "t" + str(i % 10)
    if name == "status":
        return STATUSES[i % 4]
    if name == "price":
        return "19.99"
    if name == "sku":
        return "SKU-NEW-" + str(i)
    if name == "name":
        return "name-" + str(i)
    if name == "email":
        return "user" + str(i) + "@example.com"
    if name == "city":
        return CITIES[i % 4]
    if name == "category":
        return CATEGORIES[i % 4]
    if name == "name_like":
        return "name-%"
    raise ValueError("unknown parameter generator: " + name)


def params_for(names, i, rows, worker=0):
    return [gen(n, i, rows, worker) for n in names]



def context(rows, case, iterations, ops=None):
    return {
        "rows": rows,
        "iters": iterations,
        "per_txn": case.get("per_txn", 1),
        "workers": case.get("workers", 0),
        "ops": ops if ops is not None else 0,
        "chunk": 500,
    }


def evaluate(expr, ctx):
    """Evaluate an expectation expression; only names from ctx are visible.

    A value written as `lit:<text>` is a literal, for expectations that are
    values rather than counts (a payload was renamed to `updated-0`, say).
    """
    if isinstance(expr, (int, float)):
        return str(expr)
    if isinstance(expr, str) and expr.startswith("lit:"):
        return expr[4:]
    allowed = {k: v for k, v in ctx.items()}
    try:
        value = eval(str(expr), {"__builtins__": {}}, allowed)  # noqa: S307 - our own file
    except Exception as exc:  # a malformed expression is a bug in cases.json
        raise ValueError(f"cannot evaluate expectation {expr!r}: {exc}") from exc
    if isinstance(value, float) and value.is_integer():
        value = int(value)
    return str(value)


def expected(case, ctx):
    """Expected row values for each check of a case."""
    return [[evaluate(v, ctx) for v in check.get("expect", [])] for check in case.get("checks", [])]


def compare(actual, expected_rows, case_id="", where=""):
    """Return (ok, message) for one case's check results."""
    if actual is None:
        return False, f"{case_id}: no check results from {where}"
    if len(actual) != len(expected_rows):
        return False, f"{case_id}: {where} returned {len(actual)} checks, expected {len(expected_rows)}"
    for i, (got, want) in enumerate(zip(actual, expected_rows)):
        if len(got) != len(want):
            return False, f"{case_id}: check {i} column count {len(got)} != {len(want)}"
        for g, w in zip(got, want):
            if str(g) != str(w):
                return False, f"{case_id}: check {i} gave {g!r}, expected {w!r}"
    return True, ""
