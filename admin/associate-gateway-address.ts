#!/usr/bin/env -S deno run --allow-net --allow-run --allow-read --allow-env
// SPDX-License-Identifier: MIT

import { runOrExit, parseProcessOutput, awsEc2Cmd } from "./procs.ts";
import { config as loadEnv } from "https://deno.land/x/dotenv@v0.3.0/mod.ts";
