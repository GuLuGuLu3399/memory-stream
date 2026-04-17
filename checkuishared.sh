#!/bin/bash

# Component list to check
components=(
    "ContextMenu.vue"
    "EmptyState.vue"
    "FloatingPanel.vue"
    "MarkdownViewer.vue"
    "SkeletonBlock.vue"
    "StatusBadge.vue"
)

# Check each component
for component in "${components[@]}"; do
    echo "Checking $component..."
    found=$(find frontend-workspace -name "*.vue" -not -path "*/node_modules/*" -exec grep -l "$component" {} \; 2>/dev/null)
    if [ -z "$found" ]; then
        echo "NOT FOUND: $component"
    else
        echo "FOUND in: $found"
    fi
done
