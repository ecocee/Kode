# ADR 0014: Maps have string keys, in insertion order
Status: accepted (limitation)

String keys cover JSON-like data and configuration, and keep the runtime small. Insertion order
makes output predictable (good for tests and AI agents). Integer keys are planned.
