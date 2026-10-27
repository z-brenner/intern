| Escalate when | Readiness rule | Escalated | Filename | Ready | Unsafe ready | Needless review | Readiness match | Sum inference s |
|---|---|---|---|---|---|---|---|---|
| never (2B always) | take4b | 0 | 37 | 54 | 23 | 2 | 59 | 1085 |
| always (both models on every doc) | take4b | 77 | 48 | 63 | 22 | 4 | 65 | 3814 |
| always (both models on every doc) | sticky | 77 | 48 | 48 | 14 | 9 | 54 | 3814 |
| always (both models on every doc) | agree | 77 | 48 | 27 | 4 | 17 | 40 | 3814 |
| 2B review | take4b | 23 | 41 | 69 | 31 | 1 | 70 | 1822 |
| 2B review | sticky | 23 | 41 | 54 | 23 | 6 | 59 | 1822 |
| 2B review | agree | 23 | 41 | 54 | 23 | 6 | 59 | 1822 |
| 2B parties<2 | take4b | 21 | 44 | 56 | 21 | 5 | 61 | 1818 |
| 2B parties<2 | sticky | 21 | 44 | 51 | 18 | 7 | 56 | 1818 |
| 2B parties<2 | agree | 21 | 44 | 41 | 12 | 10 | 49 | 1818 |
| 2B review OR parties<2 | take4b | 37 | 46 | 66 | 26 | 4 | 67 | 2292 |
| 2B review OR parties<2 | sticky | 37 | 46 | 51 | 18 | 9 | 56 | 2292 |
| 2B review OR parties<2 | agree | 37 | 46 | 41 | 12 | 12 | 49 | 2292 |
| 2B conf_min<0.7 | take4b | 40 | 44 | 59 | 22 | 5 | 61 | 2586 |
| 2B conf_min<0.7 | sticky | 40 | 44 | 49 | 16 | 7 | 54 | 2586 |
| 2B conf_min<0.7 | agree | 40 | 44 | 37 | 11 | 12 | 46 | 2586 |
| 2B review OR conf_min<0.7 | take4b | 47 | 46 | 64 | 24 | 4 | 65 | 2777 |
| 2B review OR conf_min<0.7 | sticky | 47 | 46 | 49 | 16 | 9 | 54 | 2777 |
| 2B review OR conf_min<0.7 | agree | 47 | 46 | 37 | 11 | 14 | 46 | 2777 |
| 2B review OR parties<2 OR conf_min<0.7 | take4b | 52 | 45 | 64 | 25 | 4 | 65 | 2934 |
| 2B review OR parties<2 OR conf_min<0.7 | sticky | 52 | 45 | 49 | 17 | 9 | 54 | 2934 |
| 2B review OR parties<2 OR conf_min<0.7 | agree | 52 | 45 | 33 | 8 | 14 | 44 | 2934 |
| 4B only | 4B | 77 | 48 | 63 | 22 | | | 2729 |

Names agree on 33/77; of those correct 28; disagree correct(4B) 20/44

2-fold CV (200 random splits): rule picked on one half, gain in (correct - 2*unsafe) over 2B-always on the other half
[(('always (both models on every doc)', 'agree'), 198), (('2B review OR conf_min<0.7', 'agree'), 1), (('2B review OR parties<2', 'agree'), 1)]
held-out gain: median 25 5% 16 95% 34
