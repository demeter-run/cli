#!/bin/sh
# Based on Deno installer: Copyright 2019 the Deno authors. All rights reserved. MIT license.
# TODO(everyone): Keep this script simple and easily auditable.

set -e

main() {
	os=$(uname -s)
	arch=$(uname -m)
	version=${1:-latest}

	root_dir="${DMTRCTL_INSTALL:-$HOME/.dmtr}"

	if [ "$version" == "latest" ]; then
		echo "using latest version"
		download_uri="https://github.com/demeter-run/cli/releases/latest/download/dmtrctl-${os}-${arch}.tar.gz"
	else
		echo "using version ${version}"
		download_uri="https://github.com/demeter-run/cli/releases/download/${version}/dmtrctl-${os}-${arch}.tar.gz"
	fi

	echo "downloading binary from ${download_uri}"

	bin_dir="$root_dir/bin"
	tmp_dir="$root_dir/tmp"
	exe="$bin_dir/dmtrctl"
	simexe="$bin_dir/dmtr"

	mkdir -p "$bin_dir"
	mkdir -p "$tmp_dir"

	curl -q --fail --location --progress-bar --output "$tmp_dir/dmtrctl.tar.gz" "$download_uri"
	
	tar -C "$tmp_dir" -xzf "$tmp_dir/dmtrctl.tar.gz"
	chmod +x "$tmp_dir/dmtrctl"
	
	# atomically rename into place:
	mv "$tmp_dir/dmtrctl" "$exe"
	#rm "$tmp_dir/dmtrctl.tar.gz"

	ln -sf $exe $simexe

	# print version
	"$exe" --version
	
	echo "dmtrctl was installed successfully to $exe"
	
	case $SHELL in
	*/zsh)
		PROFILE=$HOME/.zshrc
		PREF_SHELL=zsh
		INJECT="export PATH=\"\$PATH:$bin_dir\""
		;;
	*/bash)
		PROFILE=$HOME/.bashrc
		PREF_SHELL=bash
		INJECT="export PATH=\"\$PATH:$bin_dir\""
		;;
	*/fish)
		PROFILE=$HOME/.config/fish/config.fish
		PREF_SHELL=fish
		INJECT="fish_add_path $bin_dir"
		;;
	*/ash)
		PROFILE=$HOME/.profile
		PREF_SHELL=ash
		INJECT="export PATH=\"\$PATH:$bin_dir\""
		;;
	*)
		echo "could not detect shell, manually add ${bin_dir} to your PATH."
		exit 1
	esac

	if [[ ":$PATH:" != *":${bin_dir}:"* ]]; then
		echo >> $PROFILE && echo "$INJECT" >> $PROFILE
	fi

	echo "Detected your preferred shell is ${PREF_SHELL} and added dmtrctl to PATH."
	echo "Run 'source ${PROFILE}' or start a new terminal session to use dmtrctl."
	echo "Run 'dmtrctl --help' to get started."
}

main "$1"
