//
// Process Helper Functions
//

interface Runner {
  cmd: string[];
  stdout: number | "piped" | "inherit" | "null" | undefined;
}

/** This side-effecting procedure will cause the program to quit 
 * if the subprocess returns a non-zero code.
 */
const runOrExit = async ({ cmd, stdout }: Runner) => {
  const p = Deno.run({
    cmd,
    stdout,
  });

  const { code } = await p.status();

  if (code !== 0) {
    const rawError = await p.stderrOutput();
    const errorString = new TextDecoder().decode(rawError);
    console.log(errorString);
    Deno.exit(code);
  }

  return p;
};

const parseProcessOutput = async (p: Deno.Process) =>
  JSON.parse(new TextDecoder().decode(await p.output()));

export { runOrExit, parseProcessOutput };
