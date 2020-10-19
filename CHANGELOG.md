# NEXT: v0.9.0

- Remove Kafka-related functionalities (#375, #373, #372)
- Add docker buildkit support (#369, #389)
- Match micro-sync make move payload to system (#352, #364)
- Add log statements (#356)
- Gateway: coordinate multiplayer events using redis streams (#332, #349, #350, #351, #353, #354, #355, #388)
- Create shared model crates (#342, #339, #343, #344, #345, #346, #347, #348, #379, #380, #381, #382, #383, #384, #385, #386, #387)
- Create micro-sync (history provider) service (#331, #335, #337, #338)
- Include session-disconnected stream in game-lobby reads (#359)
- Fix memory leak in micro-changelog (#336)
- Create micro-color-chooser (#334, #365, #366, #367, #371, #377)
- Use XREADGROUP in micro-game-lobby (#358) 

# v0.8.9

- Create CHANGELOG.md
- Parameterize tinybrain model input (#328) 
fb412e0 23 hours ago (origin/main, main) Touch up browser README (#326) 
- Micro changelog : use xreadgroup  (#318), Use xgroup_create_mkstream (#319) 
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
