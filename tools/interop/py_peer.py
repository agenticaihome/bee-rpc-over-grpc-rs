#!/usr/bin/env python3
"""A Bee-RPC peer that uses the Python reference, for the interop tests.

    python -I tools/interop/py_peer.py --vectors crates/bee-rpc/tests/vectors

It listens on 127.0.0.1 on a free port and prints one line, `PORT <n>`, on
stdout. It stops when stdin closes. Service `bee.WireHarness`, all methods are
bidi streams of `buffer.Buffer`:

- Exchange: parses the request (message mode). Replies with index 0 parts: first
  a JSON summary `[[index, len, sha256], ...]` of the request parts, then each
  request part again, in order.
- Stream: the request is one part, the name of a .bee vector file. Replies with
  the Buffers of that file, unchanged.
- Skip: replies with parse_big_block.bee through the reference skip filter, with
  0.2 s between Buffers, and reads skip requests from the request direction.
- Stats: replies with one index 0 part, JSON `{"skip_chunks_sent": n}` for the
  last Skip call.
"""
import argparse
import hashlib
import json
import os
import sys
import threading
import time
from concurrent import futures

import grpc

from bee_rpc import buffer_pb2
from bee_rpc.client import parse_from_buffer, serialize_to_buffer, skip_requested_blocks
from bee_rpc.control import StreamControl
from bee_rpc.reader import read_bee_file

SERVICE = 'bee.WireHarness'
STATS = {'skip_chunks_sent': -1}


def request_parts(request_iterator):
    heads = []

    def tap(it):
        for b in it:
            if b.HasField('head'):
                heads.append(b.head.index)
            yield b

    values = list(parse_from_buffer(
        request_iterator=tap(request_iterator),
        indices={i: bytes for i in range(1, 10)},
        partitions_message_mode=True,
    ))
    return list(zip(heads, values))


def exchange(request_iterator, context):
    parts = request_parts(request_iterator)
    summary = json.dumps([[i, len(v), hashlib.sha256(v).hexdigest()] for i, v in parts]).encode()
    yield from serialize_to_buffer(message_iterator=iter([summary] + [v for _, v in parts]))


def make_stream(vectors):
    def stream(request_iterator, context):
        name = request_parts(request_iterator)[0][1].decode()
        yield from read_bee_file(os.path.join(vectors, os.path.basename(name)))
    return stream


def make_skip(vectors):
    def skip(request_iterator, context):
        control = StreamControl()
        control.watch(iter(request_iterator))
        sent = 0
        source = read_bee_file(os.path.join(vectors, 'parse_big_block.bee'))
        for b in skip_requested_blocks(source, control=control):
            if b.HasField('chunk') and len(b.chunk) > 0:
                sent += 1
            yield b
            time.sleep(0.2)
        STATS['skip_chunks_sent'] = sent
    return skip


def stats(request_iterator, context):
    for _ in request_iterator:
        pass
    yield from serialize_to_buffer(message_iterator=iter([json.dumps(STATS).encode()]))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--vectors', required=True)
    args = ap.parse_args()
    vectors = os.path.abspath(args.vectors)

    def handler(fn):
        return grpc.stream_stream_rpc_method_handler(
            fn,
            request_deserializer=buffer_pb2.Buffer.FromString,
            response_serializer=buffer_pb2.Buffer.SerializeToString)

    server = grpc.server(futures.ThreadPoolExecutor(max_workers=8))
    server.add_generic_rpc_handlers((grpc.method_handlers_generic_handler(SERVICE, {
        'Exchange': handler(exchange),
        'Stream': handler(make_stream(vectors)),
        'Skip': handler(make_skip(vectors)),
        'Stats': handler(stats),
    }),))
    port = server.add_insecure_port('127.0.0.1:0')
    server.start()
    print('PORT %d' % port, flush=True)

    def wait_stdin():
        sys.stdin.read()
        server.stop(0)

    threading.Thread(target=wait_stdin, daemon=True).start()
    server.wait_for_termination()


if __name__ == '__main__':
    main()
