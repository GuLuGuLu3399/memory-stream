#!/bin/bash

# Component list to check
components=(
    "App.vue"
    "BacklinksPanel.vue"
    "BottomNav.vue"
    "DetailDrawer.vue"
    "EntranceAnimation.vue"
    "FloatingCommandBar.vue"
    "FloatingCompass.vue"
    "FlowReader.vue"
    "LeftDock.vue"
    "RightDock.vue"
    "SearchBar.vue"
    "StatsWidget.vue"
    "TimelineTrack.vue"
    "ZenReader.vue"
    "DetailDrawerContent.vue"
    "DetailDrawerFooter.vue"
    "ZenEdgeHandle.vue"
    "ZenSealButton.vue"
    "TocNode.vue"
    "CardNode.vue"
    "SkeletonLine.vue"
    "GraphView.vue"
    "ListView.vue"
    "DateColumn.vue"
    "ListCardRow.vue"
    "ListViewHeader.vue"
    "SpineNode.vue"
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
