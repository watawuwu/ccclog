#!/bin/sh -l

repo_path="$1"
revision_spec="$2"

changelog=$(ccclog  $repo_path $revision_spec)
changelog="${changelog//$'\n'/'%0A'}"
echo "::set-output name=changelog::$changelog"
