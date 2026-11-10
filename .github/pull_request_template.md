<!--
Title: <type>[optional scope]: <description> #<issue-number>
The title ends with the issue reference as a bare `#<issue-number>` token; the issue reference never appears in the Conventional Commits scope, and the pull request's own reference belongs in this body.
-->

## Summary

<!-- What changed and why. -->

## Issue

Closes: #<issue-number>

## Verification

<!-- Commands run and results observed. -->

## Checklist

- [ ] The title ends with the bare `#<issue-number>` issue reference.
- [ ] The issue reference is absent from the Conventional Commits scope.
- [ ] This body carries the `Closes: #<issue-number>` closing keyword.
- [ ] The declared `forge` selects the platform helper; no platform is hard-coded.
