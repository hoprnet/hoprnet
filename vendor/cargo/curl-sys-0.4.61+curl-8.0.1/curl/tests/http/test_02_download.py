#!/usr/bin/env python3
# -*- coding: utf-8 -*-
#***************************************************************************
#                                  _   _ ____  _
#  Project                     ___| | | |  _ \| |
#                             / __| | | | |_) | |
#                            | (__| |_| |  _ <| |___
#                             \___|\___/|_| \_\_____|
#
# Copyright (C) Daniel Stenberg, <daniel@haxx.se>, et al.
#
# This software is licensed as described in the file COPYING, which
# you should have received as part of this distribution. The terms
# are also available at https://curl.se/docs/copyright.html.
#
# You may opt to use, copy, modify, merge, publish, distribute and/or sell
# copies of the Software, and permit persons to whom the Software is
# furnished to do so, under the terms of the COPYING file.
#
# This software is distributed on an "AS IS" basis, WITHOUT WARRANTY OF ANY
# KIND, either express or implied.
#
# SPDX-License-Identifier: curl
#
###########################################################################
#
import difflib
import filecmp
import logging
import os
import pytest

from testenv import Env, CurlClient


log = logging.getLogger(__name__)


@pytest.mark.skipif(condition=Env.setup_incomplete(),
                    reason=f"missing: {Env.incomplete_reason()}")
class TestDownload:

    @pytest.fixture(autouse=True, scope='class')
    def _class_scope(self, env, httpd, nghttpx):
        if env.have_h3():
            nghttpx.start_if_needed()
        httpd.clear_extra_configs()
        httpd.reload()

    @pytest.fixture(autouse=True, scope='class')
    def _class_scope(self, env, httpd):
        env.make_data_file(indir=httpd.docs_dir, fname="data-100k", fsize=100*1024)
        env.make_data_file(indir=httpd.docs_dir, fname="data-1m", fsize=1024*1024)
        env.make_data_file(indir=httpd.docs_dir, fname="data-10m", fsize=10*1024*1024)

    # download 1 file
    @pytest.mark.parametrize("proto", ['http/1.1', 'h2', 'h3'])
    def test_02_01_download_1(self, env: Env, httpd, nghttpx, repeat, proto):
        if proto == 'h3' and not env.have_h3():
            pytest.skip("h3 not supported")
        curl = CurlClient(env=env)
        url = f'https://{env.authority_for(env.domain1, proto)}/data.json'
        r = curl.http_download(urls=[url], alpn_proto=proto)
        assert r.exit_code == 0, f'{r}'
        r.check_stats(count=1, exp_status=200)

    # download 2 files
    @pytest.mark.parametrize("proto", ['http/1.1', 'h2', 'h3'])
    def test_02_02_download_2(self, env: Env, httpd, nghttpx, repeat, proto):
        if proto == 'h3' and not env.have_h3():
            pytest.skip("h3 not supported")
        curl = CurlClient(env=env)
        url = f'https://{env.authority_for(env.domain1, proto)}/data.json?[0-1]'
        r = curl.http_download(urls=[url], alpn_proto=proto)
        assert r.exit_code == 0
        r.check_stats(count=2, exp_status=200)

    # download 100 files sequentially
    @pytest.mark.parametrize("proto", ['http/1.1', 'h2', 'h3'])
    def test_02_03_download_100_sequential(self, env: Env,
                                           httpd, nghttpx, repeat, proto):
        if proto == 'h3' and not env.have_h3():
            pytest.skip("h3 not supported")
        curl = CurlClient(env=env)
        urln = f'https://{env.authority_for(env.domain1, proto)}/data.json?[0-99]'
        r = curl.http_download(urls=[urln], alpn_proto=proto)
        assert r.exit_code == 0
        r.check_stats(count=100, exp_status=200)
        # http/1.1 sequential transfers will open 1 connection
        assert r.total_connects == 1

    # download 100 files parallel
    @pytest.mark.parametrize("proto", ['http/1.1', 'h2', 'h3'])
    def test_02_04_download_100_parallel(self, env: Env,
                                         httpd, nghttpx, repeat, proto):
        if proto == 'h3' and not env.have_h3():
            pytest.skip("h3 not supported")
        max_parallel = 6 if proto == 'http/1.1' else 50
        curl = CurlClient(env=env)
        urln = f'https://{env.authority_for(env.domain1, proto)}/data.json?[0-99]'
        r = curl.http_download(urls=[urln], alpn_proto=proto, extra_args=[
            '--parallel', '--parallel-max', f'{max_parallel}'
        ])
        assert r.exit_code == 0
        r.check_stats(count=100, exp_status=200)
        if proto == 'http/1.1':
            # http/1.1 parallel transfers will open multiple connections
            assert r.total_connects > 1
        else:
            # http2 parallel transfers will use one connection (common limit is 100)
            assert r.total_connects == 1

    # download 500 files sequential
    @pytest.mark.parametrize("proto", ['http/1.1', 'h2', 'h3'])
    def test_02_05_download_500_sequential(self, env: Env,
                                           httpd, nghttpx, repeat, proto):
        if proto == 'h3' and not env.have_h3():
            pytest.skip("h3 not supported")
        curl = CurlClient(env=env)
        urln = f'https://{env.authority_for(env.domain1, proto)}/data.json?[0-499]'
        r = curl.http_download(urls=[urln], alpn_proto=proto)
        assert r.exit_code == 0
        r.check_stats(count=500, exp_status=200)
        if proto == 'http/1.1':
            # http/1.1 parallel transfers will open multiple connections
            assert r.total_connects > 1
        else:
            # http2 parallel transfers will use one connection (common limit is 100)
            assert r.total_connects == 1

    # download 500 files parallel
    @pytest.mark.parametrize("proto", ['h2', 'h3'])
    def test_02_06_download_500_parallel(self, env: Env,
                                         httpd, nghttpx, repeat, proto):
        if proto == 'h3' and not env.have_h3():
            pytest.skip("h3 not supported")
        count = 500
        max_parallel = 50
        curl = CurlClient(env=env)
        urln = f'https://{env.authority_for(env.domain1, proto)}/data.json?[000-{count-1}]'
        r = curl.http_download(urls=[urln], alpn_proto=proto, extra_args=[
            '--parallel', '--parallel-max', f'{max_parallel}'
        ])
        assert r.exit_code == 0
        r.check_stats(count=count, exp_status=200)
        # http2 parallel transfers will use one connection (common limit is 100)
        assert r.total_connects == 1

    # download files parallel, check connection reuse/multiplex
    @pytest.mark.parametrize("proto", ['h2', 'h3'])
    def test_02_07_download_reuse(self, env: Env,
                                  httpd, nghttpx, repeat, proto):
        if proto == 'h3' and not env.have_h3():
            pytest.skip("h3 not supported")
        count = 200
        curl = CurlClient(env=env)
        urln = f'https://{env.authority_for(env.domain1, proto)}/data.json?[0-{count-1}]'
        r = curl.http_download(urls=[urln], alpn_proto=proto,
                               with_stats=True, extra_args=[
            '--parallel', '--parallel-max', '200'
        ])
        assert r.exit_code == 0, f'{r}'
        r.check_stats(count=count, exp_status=200)
        # should have used 2 connections only (test servers allow 100 req/conn)
        assert r.total_connects == 2, "h2 should use fewer connections here"

    # download files parallel with http/1.1, check connection not reused
    @pytest.mark.parametrize("proto", ['http/1.1'])
    def test_02_07b_download_reuse(self, env: Env,
                                   httpd, nghttpx, repeat, proto):
        count = 20
        curl = CurlClient(env=env)
        urln = f'https://{env.authority_for(env.domain1, proto)}/data.json?[0-{count-1}]'
        r = curl.http_download(urls=[urln], alpn_proto=proto,
                               with_stats=True, extra_args=[
            '--parallel'
        ])
        assert r.exit_code == 0, f'{r}'
        r.check_stats(count=count, exp_status=200)
        # http/1.1 should have used count connections
        assert r.total_connects == count, "http/1.1 should use this many connections"

    @pytest.mark.parametrize("proto", ['http/1.1', 'h2', 'h3'])
    def test_02_08_1MB_serial(self, env: Env,
                              httpd, nghttpx, repeat, proto):
        if proto == 'h3' and not env.have_h3():
            pytest.skip("h3 not supported")
        count = 20
        urln = f'https://{env.authority_for(env.domain1, proto)}/data-1m?[0-{count-1}]'
        curl = CurlClient(env=env)
        r = curl.http_download(urls=[urln], alpn_proto=proto)
        assert r.exit_code == 0
        r.check_stats(count=count, exp_status=200)

    @pytest.mark.parametrize("proto", ['http/1.1', 'h2', 'h3'])
    def test_02_09_1MB_parallel(self, env: Env,
                              httpd, nghttpx, repeat, proto):
        if proto == 'h3' and not env.have_h3():
            pytest.skip("h3 not supported")
        count = 20
        urln = f'https://{env.authority_for(env.domain1, proto)}/data-1m?[0-{count-1}]'
        curl = CurlClient(env=env)
        r = curl.http_download(urls=[urln], alpn_proto=proto, extra_args=[
            '--parallel'
        ])
        assert r.exit_code == 0
        r.check_stats(count=count, exp_status=200)

    @pytest.mark.parametrize("proto", ['http/1.1', 'h2', 'h3'])
    def test_02_10_10MB_serial(self, env: Env,
                              httpd, nghttpx, repeat, proto):
        if proto == 'h3' and not env.have_h3():
            pytest.skip("h3 not supported")
        count = 20
        urln = f'https://{env.authority_for(env.domain1, proto)}/data-10m?[0-{count-1}]'
        curl = CurlClient(env=env)
        r = curl.http_download(urls=[urln], alpn_proto=proto)
        assert r.exit_code == 0
        r.check_stats(count=count, exp_status=200)

    @pytest.mark.parametrize("proto", ['http/1.1', 'h2', 'h3'])
    def test_02_11_10MB_parallel(self, env: Env,
                              httpd, nghttpx, repeat, proto):
        if proto == 'h3' and not env.have_h3():
            pytest.skip("h3 not supported")
        count = 20
        urln = f'https://{env.authority_for(env.domain1, proto)}/data-10m?[0-{count-1}]'
        curl = CurlClient(env=env)
        r = curl.http_download(urls=[urln], alpn_proto=proto, extra_args=[
            '--parallel'
        ])
        assert r.exit_code == 0
        r.check_stats(count=count, exp_status=200)

    @pytest.mark.parametrize("proto", ['h2', 'h3'])
    def test_02_12_head_serial_https(self, env: Env,
                                     httpd, nghttpx, repeat, proto):
        if proto == 'h3' and not env.have_h3():
            pytest.skip("h3 not supported")
        count = 100
        urln = f'https://{env.authority_for(env.domain1, proto)}/data-10m?[0-{count-1}]'
        curl = CurlClient(env=env)
        r = curl.http_download(urls=[urln], alpn_proto=proto, extra_args=[
            '--head'
        ])
        assert r.exit_code == 0
        r.check_stats(count=count, exp_status=200)

    @pytest.mark.parametrize("proto", ['h2'])
    def test_02_13_head_serial_h2c(self, env: Env,
                                    httpd, nghttpx, repeat, proto):
        if proto == 'h3' and not env.have_h3():
            pytest.skip("h3 not supported")
        count = 100
        urln = f'http://{env.domain1}:{env.http_port}/data-10m?[0-{count-1}]'
        curl = CurlClient(env=env)
        r = curl.http_download(urls=[urln], alpn_proto=proto, extra_args=[
            '--head', '--http2-prior-knowledge', '--fail-early'
        ])
        assert r.exit_code == 0
        r.check_stats(count=count, exp_status=200)

    def test_02_20_h2_small_frames(self, env: Env, httpd, repeat):
        # Test case to reproduce content corruption as observed in
        # https://github.com/curl/curl/issues/10525
        # To reliably reproduce, we need an Apache httpd that supports
        # setting smaller frame sizes. This is not released yet, we
        # test if it works and back out if not.
        httpd.set_extra_config(env.domain1, lines=[
            f'H2MaxDataFrameLen 1024',
        ])
        assert httpd.stop()
        if not httpd.start():
            # no, not supported, bail out
            httpd.set_extra_config(env.domain1, lines=None)
            assert httpd.start()
            pytest.skip(f'H2MaxDataFrameLen not supported')
        # ok, make 100 downloads with 2 parallel running and they
        # are expected to stumble into the issue when using `lib/http2.c`
        # from curl 7.88.0
        count = 100
        urln = f'https://{env.authority_for(env.domain1, "h2")}/data-1m?[0-{count-1}]'
        curl = CurlClient(env=env)
        r = curl.http_download(urls=[urln], alpn_proto="h2", extra_args=[
            '--parallel', '--parallel-max', '2'
        ])
        assert r.exit_code == 0
        r.check_stats(count=count, exp_status=200)
        srcfile = os.path.join(httpd.docs_dir, 'data-1m')
        for i in range(count):
            dfile = curl.download_file(i)
            assert os.path.exists(dfile)
            if not filecmp.cmp(srcfile, dfile, shallow=False):
                diff = "".join(difflib.unified_diff(a=open(srcfile).readlines(),
                                                    b=open(dfile).readlines(),
                                                    fromfile=srcfile,
                                                    tofile=dfile,
                                                    n=1))
                assert False, f'download {dfile} differs:\n{diff}'
        # restore httpd defaults
        httpd.set_extra_config(env.domain1, lines=None)
        assert httpd.stop()
        assert httpd.start()

