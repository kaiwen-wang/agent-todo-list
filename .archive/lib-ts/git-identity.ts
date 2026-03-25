/**
 * Read git user identity from local git config.
 * Used for "me" resolution in member lookups and init defaults.
 */

export interface GitIdentity {
  name: string | null;
  email: string | null;
}

/** Read git config user.name and user.email. Returns nulls if not configured. */
export function getGitIdentity(): GitIdentity {
  const nameResult = Bun.spawnSync(["git", "config", "user.name"], {
    stdout: "pipe",
    stderr: "pipe",
  });
  const emailResult = Bun.spawnSync(["git", "config", "user.email"], {
    stdout: "pipe",
    stderr: "pipe",
  });

  const name = nameResult.exitCode === 0 ? nameResult.stdout.toString().trim() || null : null;
  const email = emailResult.exitCode === 0 ? emailResult.stdout.toString().trim() || null : null;

  return { name, email };
}
