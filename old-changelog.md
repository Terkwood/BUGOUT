# v1.2.0

Add "new game" button at end of game. #271, #462

# v1.1.0

Add two more difficulty settings for play against KataGo.  #454, #457

# v1.0.4

Force tinybrain to request Komi 6.5.  #225, #455

# v1.0.3

Address GDPR.  #453, #452, #416 

# v1.0.2

- Fix spammy dialog during reconnection to AI play. #450, #238
- Update webpack dep in browser. #451

# v1.0.1

Adds a link to the Github repository on the front page.  #447, #448

# v1.0.0

Welcome to the v1.0.0 release of BUGOUT!

We welcome you to play online at [go.terkwood.farm](https://go.terkwood.farm).

This release allows you to play against a much faster version of KataGo. The
search space for the AI is limited, but it's still going to be a difficult
opponent for most players.

- Provide two settings for playing KataGo (#322, #432, #442, #441, #439, #437, #435)
- Use [prettier](https://prettier.io/) to format the browser code (#431)
- Upgrades shared structs in botlink (#429, #406)

# v0.9.1

- gateway: fix deprecated time call (#426, #413)
- Use XREADGROUP in gateway & botlink, upgrade their redis dep (#302, #310, #420, #423, #427)
- Simplify redis-streams crate (#421)

# v0.9.0

This release focuses on providing multiplayer functionality through a memory-efficent design focusing on Redis and Rust. (#174)

- Clean up admin scripts to account for reduced footprint (#404, #403, #399, #398, #397, #396)
- Remove Kafka-related functionalities (#375, #373, #372)
- Add docker buildkit support (#369, #389)
- Match micro-sync make move payload to system (#352, #364)
- Alter log statements (#356, #414)
- Gateway: coordinate multiplayer events using redis streams (#332, #349, #350, #351, #353, #354, #355, #388, #394, #405)
- Create shared model crates (#342, #339, #343, #344, #345, #346, #347, #348, #379, #380, #381, #382, #383, #384, #385, #386, #387, #407)
- Create micro-sync (history provider) service (#331, #335, #337, #338, #410)
- Include session-disconnected stream in game-lobby reads (#359)
- Fix memory leak in micro-changelog (#336)
- Create micro-color-chooser (#334, #365, #366, #367, #371, #377)
- Use XREADGROUP in micro-game-lobby (#358)

# v0.8.9

- Create CHANGELOG.md
- Parameterize tinybrain model input (#328)
  fb412e0 23 hours ago (origin/main, main) Touch up browser README (#326)
- Micro changelog : use xreadgroup (#318), Use xgroup_create_mkstream (#319)
- Fix branch dependency
- Remove entry ID repo from micro-judge streams (#312)
- Add wake script
- admin nits: XL builder, desc script (#311)
- Update .env example
- Add elastic-IP association script... (#309)
- Admin script: cleanup dev env (#308)
- Create CODE_OF_CONDUCT.md
- Create CONTRIBUTING.md
- Improve tinybrain systemd script and example (#307)
- Add micro game lobby (#184)
- Lock in redis streams versions as 0.2.0 (#306)
- Add AMI cleanup script (#303)
- Update images
- Update FCOS image version
