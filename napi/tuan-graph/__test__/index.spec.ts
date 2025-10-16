import test from 'ava'

import { typescript } from '../index'

test('create typescript graph', (t) => {
  const graph = typescript.getGraph("/Users/arthur-fontaine/Developer/code/github.com/arthur-fontaine/agrume")

  t.truthy(graph.nodes.length > 0)
  t.truthy(graph.edges.length > 0)
})

test('positioning', (t) => {
  const graph = typescript.getGraph("/Users/arthur-fontaine/Developer/code/github.com/arthur-fontaine/agrume")
  graph.positioning()

  for (const node of graph.nodes) {
    const [x, y] = node.position
    t.truthy(typeof x === 'number')
    t.truthy(x !== 0)
    t.truthy(typeof y === 'number')
    t.truthy(y !== 0)
  }
})

test('clusterize', (t) => {
  const graph = typescript.getGraph("/Users/arthur-fontaine/Developer/code/github.com/arthur-fontaine/agrume")
  const clusters = graph.clusterize(100)

  t.truthy(clusters.length > 0)

  for (const cluster of clusters) {
    t.truthy(cluster.members.length > 0)
  }
})
