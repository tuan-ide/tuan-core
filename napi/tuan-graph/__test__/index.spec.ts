import path from 'node:path';
import { execSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import anyTest, { type TestFn } from 'ava'

import { typescript } from '../index'

const test = anyTest as TestFn<{ projectDir: string }>;

test.before((t) => {
  const repoUrl = 'https://github.com/arthur-fontaine/agrume.git';
  const cloneDir = path.join(tmpdir(), `agrume-${Date.now()}-${Math.random().toString(16).slice(2)}`);

  execSync(`git clone ${repoUrl} ${cloneDir}`, { stdio: 'inherit' });
  execSync(`pnpm install`, { stdio: 'inherit', cwd: cloneDir });
  execSync(`pnpm build`, { stdio: 'inherit', cwd: cloneDir });

  t.context.projectDir = cloneDir;
});

test('create typescript graph', (t) => {
  const graph = typescript.getGraph(t.context.projectDir)

  t.truthy(graph.nodes.length > 0)
  t.truthy(graph.edges.length > 0)
})

test('positioning', (t) => {
  const graph = typescript.getGraph(t.context.projectDir)
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
  const graph = typescript.getGraph(t.context.projectDir)
  const clusters = graph.clusterize(100)

  t.truthy(clusters.length > 0)

  for (const cluster of clusters) {
    t.truthy(cluster.members.length > 0)
  }
})
