# ADR 0007: Return types are written when a function returns a value
Status: accepted (changes the first syntax draft, which inferred them)

The signature is the contract. It makes errors point at the right place, keeps the type checker
small, and gives AI agents certainty. Functions that return nothing omit `->`.
