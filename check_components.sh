#!/bin/bash

# Component list to check
components=(
    "RightAstrolabe.vue"
    "TheForge.vue"
    "TitleBar.vue"
    "BacklinksRadar.vue"
    "ForgeEmptyState.vue"
    "ForgeHeader.vue"
    "ForgePreview.vue"
    "MergeActionBar.vue"
    "MergeBlastRadius.vue"
    "MergeSurvivorColumn.vue"
    "MergeVictimsColumn.vue"
    "CategoryRibbon.vue"
    "SidebarCardItem.vue"
    "TabSelector.vue"
    "ImportPanel.vue"
    "Settings.vue"
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
