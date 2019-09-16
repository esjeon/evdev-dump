
evdev-dump
==========

A simple utility for dumping evdev events, written in Rust.


Usage
-----

It's simple:

	# evdev-dump /dev/input/eventXX

Note that you will need root privilege in most cases.

While running, double-press Q to exit the program.


TODO
----
 * Handle signals (restore ECHO!)
 * An option for dump ALL devices?

