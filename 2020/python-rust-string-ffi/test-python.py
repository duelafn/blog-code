#!/usr/bin/env python3
# -*- coding: utf-8 -*-
# SPDX-License-Identifier: MIT
import json
import unittest

from cffi import FFI
ffi = FFI()

import platform
if 'Windows' == platform.system():
    libmylib = ffi.dlopen('./target/release/libmylib.dll')
else:
    libmylib = ffi.dlopen('./target/release/libmylib.so')

ffi.cdef('''
void mylib_free_string(const char *n);
char* mylib_myfunc_str(const char *n);
''')

# Split myfunc_raw from myfunc so that I can test invalid inputs
def myfunc_raw(req):
    rstr = ffi.NULL
    try:
        rstr = libmylib.mylib_myfunc_str(req)
        if rstr == ffi.NULL:
            return None
        return ffi.string(rstr)
    finally:
        if rstr != ffi.NULL:
            libmylib.mylib_free_string(rstr)
    return None

def myfunc(req):
    res = myfunc_raw(json.dumps(req).encode("UTF-8"))
    return None if res is None else json.loads(res.decode('UTF-8'))


class MyTest(unittest.TestCase):
    def test_ffi(self):
        rv = myfunc({ "plugh": "A test string" })
        self.assertIsNone(rv.get('Err'))
        self.assertEqual(rv.get('Ok'), 'plugh has length 13')

        rv = myfunc({ "foo": "A test string" })
        self.assertIsNone(rv.get('Ok'))
        self.assertEqual(rv.get('Err'), 'plugh not present or not valid')

    def test_invalid_json(self):
        rv = json.loads(myfunc_raw("{Invalid json}".encode("UTF-8")).decode('UTF-8'))
        self.assertTrue(rv.get('Err').startswith('JSON Parse error:'))

    def test_invalid_utf8(self):
        rv = json.loads(myfunc_raw(b"\xE7").decode('UTF-8'))
        self.assertTrue(rv.get('Err').startswith('Encoding error:'))


if __name__ == '__main__':
    unittest.main()
