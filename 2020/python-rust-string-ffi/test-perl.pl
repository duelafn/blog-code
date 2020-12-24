#!/usr/bin/perl
use strict; use warnings; use 5.020;
use utf8;
use Test::More;
use JSON;

use FFI::Platypus;
my $FFI = FFI::Platypus->new(api => 1);
my $mylib_so = ($^O eq "MSWin32") ? "libmylib.dll" : "libmylib.so";
$FFI->lib("target/release/$mylib_so");
$FFI->attach(mylib_myfunc_str => ['string'] => 'opaque');
$FFI->attach(mylib_free_string => ['opaque'] => 'void');

sub myfunc_raw {
    my ($ptr, $str) = eval {
        my $p = mylib_myfunc_str($_[0]);
        ($p, $FFI->cast('opaque' => 'string', $p))
    };
    mylib_free_string($ptr) if $ptr;
    die if $@;
    return $str;
}

sub myfunc {
    my $rv = myfunc_raw(encode_json($_[0]));
    return $rv ? decode_json($rv) : undef;
}

my $rv;

$rv = myfunc({ "plugh" => "A test string" });
ok !defined($$rv{Err});
is $$rv{Ok}, 'plugh has length 13';

$rv = myfunc({ "foo" => "A test string" });
ok !defined($$rv{Ok});
is $$rv{Err}, 'plugh not present or not valid';

$rv = decode_json(myfunc_raw("{Invalid json}"));
ok $$rv{Err} =~ /^JSON Parse error:/;

$rv = decode_json(myfunc_raw("\xE7"));
ok $$rv{Err} =~ /^Encoding error:/;


done_testing;
