[简体中文](./CONTRIBUTING_zh-Hans.md)


```shell
TypeError: Jest: Got error running globalSetup - /home/gumayusi/projects/cryptape.com/axon/tests/e2e/jest/setup.js, reason: Cannot read properties of undefined (reading 'wsEndpoint')                                                 
    at setup (/home/gumayusi/projects/cryptape.com/axon/tests/e2e/jest/setup.js:92:47)                                                                    
    at processTicksAndRejections (node:internal/process/task_queues:96:5)
    at async /home/gumayusi/projects/cryptape.com/axon/tests/e2e/node_modules/@jest/core/build/runGlobalHook.js:109:13                                    
    at async waitForPromiseWithCleanup (/home/gumayusi/projects/cryptape.com/axon/tests/e2e/node_modules/@jest/transform/build/ScriptTransformer.js:160:5)
    at async ScriptTransformer.requireAndTranspileModule (/home/gumayusi/projects/cryptape.com/axon/tests/e2e/node_modules/@jest/transform/build/ScriptTransformer.js:782:16)                                                          
    at async runGlobalHook (/home/gumayusi/projects/cryptape.com/axon/tests/e2e/node_modules/@jest/core/build/runGlobalHook.js:101:9)                     
    at async runJest (/home/gumayusi/projects/cryptape.com/axon/tests/e2e/node_modules/@jest/core/build/runJest.js:317:5)                                 
    at async _run10000 (/home/gumayusi/projects/cryptape.com/axon/tests/e2e/node_modules/@jest/core/build/cli/index.js:326:7)                             
    at async runCLI (/home/gumayusi/projects/cryptape.com/axon/tests/e2e/node_modules/@jest/core/build/cli/index.js:191:3)                                
    at async Object.run (/home/gumayusi/projects/cryptape.com/axon/tests/e2e/node_modules/jest-cli/build/run.js:124:37)                                   
error Command failed with exit code 1.
info Visit https://yarnpkg.com/en/docs/cli/run for documentation about this command.

```
