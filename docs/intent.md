# Intent System

The Intent system parses high-level user commands into actionable discrete steps.

## Pipeline
1. **Parser**: Translates natural language into a canonical `IntentResult`.
2. **Planner**: Breaks the `IntentResult` into a sequence of atomic `Step` actions.
3. **Executor**: Carries out each `Step` sequentially, performing web navigation, shell commands, or AI operations.
