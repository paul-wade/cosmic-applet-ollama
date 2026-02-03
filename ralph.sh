#!/bin/bash
# Ralph Loop - Autonomous AI Development
# Based on Geoffrey Huntley's Ralph technique
# https://ghuntley.com/ralph/

set -e

MAX_ITERATIONS=${1:-50}
COMPLETION_PROMISE="RALPH_COMPLETE"
PROMPT_FILE="ralph-prompt.md"
LOG_FILE="ralph.log"

echo "🔄 Starting Ralph Loop (max $MAX_ITERATIONS iterations)"
echo "📝 Prompt: $PROMPT_FILE"
echo "✅ Completion: $COMPLETION_PROMISE"
echo "📋 Log: $LOG_FILE"
echo ""

iteration=0
while [ $iteration -lt $MAX_ITERATIONS ]; do
    iteration=$((iteration + 1))
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "🔁 Iteration $iteration / $MAX_ITERATIONS"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    # Run Claude Code with the prompt file
    output=$(claude --print "$(cat $PROMPT_FILE)" 2>&1) || true

    # Log output
    echo "=== Iteration $iteration ===" >> "$LOG_FILE"
    echo "$output" >> "$LOG_FILE"
    echo "" >> "$LOG_FILE"

    # Check for completion
    if echo "$output" | grep -q "$COMPLETION_PROMISE"; then
        echo ""
        echo "🎉 RALPH COMPLETE!"
        echo "✅ Completion promise found after $iteration iterations"
        exit 0
    fi

    # Brief pause between iterations
    sleep 2
done

echo ""
echo "⚠️  Max iterations ($MAX_ITERATIONS) reached without completion"
echo "Check $LOG_FILE for details"
exit 1
