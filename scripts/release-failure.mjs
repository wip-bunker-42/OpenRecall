import { notifyOnce } from './github.mjs';
const tag = process.env.RELEASE_TAG ?? '';
if (!/^v\d+\.\d+\.\d+$/.test(tag)) throw new Error('Invalid release notification tag');
await notifyOnce({
  marker: `<!-- openrecall-release-failure:${tag} -->`,
  label: 'release-failure',
  title: `Release ${tag} did not complete`,
  body: `The verified release workflow failed. No incomplete release should be published. Inspect the failed job and retry after fixing its cause.\n\n[Workflow run](https://github.com/wip-bunker-42/OpenRecall/actions/runs/${process.env.GITHUB_RUN_ID})`,
});
