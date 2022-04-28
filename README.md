# synacor-challenge
Rust implementation of the [Synacor Challenge](https://challenge.synacor.com/)

Usage:

		synacor-challenge.exe [OPTIONS] <INPUT>
		or
		cargo run -- [OPTIONS] <INPUT>

	ARGS:
		<INPUT>    Your challenge.bin file

	OPTIONS:
		-d <FILE>               Export a decompiled version of the challenge binary to text file
		-h, --help              Print help information
		-i                      Disables autosolving and runs the challenge binary in interactive
								terminal mode.
		-t <SEARCH_TYPE>        Enables the search for teleporter setting rather than using a
								precomputed solution. [possible values: single, parallel]

Example:

	synacor-challenge.exe challenge.bin
	
Thank you to **Eric Wastl** for a fun set of challenges!