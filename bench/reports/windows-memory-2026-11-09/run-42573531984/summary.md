Windows memory probe: 4 logical processors, 4 server threads, 16378.7 MiB physical; MIN_AVAILABLE_FOR_ACCURATE 8192 MiB.

| Loaded | Stage | Available | Working set | Shipped credit | Plan sees | Accurate model | Credit = working set | Plan sees | Accurate model |
| --- | --- | ---: | ---: | ---: | ---: | --- | ---: | ---: | --- |
| accurate | after-health | 5381.5 | 7895.7 | 2883 | 8264.5 | fits | 7895.7 | 13277.2 | fits |
| accurate | after-request | 5409.7 | 8109.5 | 2883 | 8292.7 | fits | 8109.5 | 13519.2 | fits |
| accurate | after-idle | 6080.8 | 7528.3 | 2883 | 8963.8 | fits | 7528.3 | 13609.1 | fits |
| fast | after-health | 9350.6 | 4318.6 | 2086 | 11436.6 | fits | 4318.6 | 13669.2 | fits |
| fast | after-request | 9164.9 | 4513.4 | 2086 | 11250.9 | fits | 4513.4 | 13678.3 | fits |
| fast | after-idle | 9194.3 | 4513.4 | 2086 | 11280.3 | fits | 4513.4 | 13707.7 | fits |

| Model | Available before load | After request | Taken by the load | Given back 3 s after exit | Shipped credit | Under-credit |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| accurate | 13204.9 | 5409.7 | 7795.2 | 7578.4 | 2883 | 4695.4 |
| fast | 13669.2 | 9164.9 | 4504.3 | 4515.2 | 2086 | 2429.2 |

With the shipped credit, a machine with A MiB available before any model is loaded:
- loads the accurate model for the first document when A >= 8192;
- with the accurate model loaded, plans the fast one for the next document when A < 13104.2;
- with the fast model loaded, plans the accurate one again when A >= 10610.3.
So the models swap before every document for 10610 <= A < 13104.2 MiB.
And for 8192 <= A < 10610.3 MiB the accurate model reads one document and the fast one the rest.
This runner had A = 13204.9 MiB before the first load.
