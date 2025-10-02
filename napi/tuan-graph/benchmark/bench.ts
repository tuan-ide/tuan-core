import { Bench } from 'tinybench'

import { typescript } from '../index.js'

const b = new Bench()

b.add('create typescript graph', async () => {
  const graph = typescript.getGraph("/Users/arthur-fontaine/Developer/code/github.com/arthur-fontaine/agrume")
  graph.positioning()
})

await b.run()

console.table(b.table())
