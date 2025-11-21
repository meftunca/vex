#!/usr/bin/env bash
set -euo pipefail

echo "Scanning borrow_checker modules for check_expression/check_statement parent_span forwarding..."
ROOT_DIR=$(dirname "$(dirname "${BASH_SOURCE[0]}")")
REPO_DIR="$ROOT_DIR"

find "$REPO_DIR/vex-compiler/src/borrow_checker" -type f -name "*.rs" -print0 | while IFS= read -r -d '' file; do
    # Find check_expression / check_expression_for_borrows / check_statement invocations
    grep -n -E "check_expression\(|check_expression_for_borrows\(|check_statement\(" "$file" || true | while IFS= read -r line; do
        # If a call contains 'parent_span' or 'this_span' or 'span_id.as_ref().or(parent_span)' we consider it OK
        if echo "$line" | grep -q "parent_span\|this_span\|span_id.as_ref().or(parent_span)\|Some(&parent_span)"; then
            : # OK
        else
            echo "Potential missing parent_span forwarding in $file: $line"
        fi
    done
done

echo "Scan completed"

exit 0
