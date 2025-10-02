import test from 'ava'

import { typescript } from '../index'

test('create typescript graph', (t) => {
  const graph = typescript.getGraph("/Users/arthur-fontaine/Developer/code/github.com/arthur-fontaine/agrume")
  graph.positioning()

  t.truthy(graph.nodes.length > 0)
  t.truthy(graph.edges.length > 0)
})
