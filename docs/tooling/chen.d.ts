/**
 * TypeScript definitions for Chen Lang runtime globals.
 *
 * JS-like Chen source files (.chen.js) are compatible with JavaScript tooling
 * for syntax highlighting and linting. Using these definitions in your editor
 * (like VS Code) will provide better IntelliSense for Chen-specific built-ins.
 */

/**
 * Global Chen namespace for runtime specific capabilities.
 */
declare namespace Chen {
  /**
   * File system operations.
   */
  namespace fs {
    /**
     * Reads a file and returns its content as a string.
     */
    function readTextFile(path: string): string;
    /**
     * Writes content to a file.
     */
    function writeTextFile(path: string, content: string): void;
    /**
     * Checks if a path exists.
     */
    function exists(path: string): boolean;
    /**
     * Removes a file or directory.
     */
    function remove(path: string): void;
    /**
     * Reads a directory and returns its entries as an array of strings.
     */
    function readDir(path: string): string[];

    /** @deprecated use readTextFile */
    function read_file(path: string): string;
    /** @deprecated use writeTextFile */
    function write_file(path: string, content: string): void;
    /** @deprecated use readDir */
    function read_dir(path: string): string[];
  }

  /**
   * HTTP client.
   */
  namespace http {
    interface Response {
      status: number;
      body: string;
      headers: { [key: string]: string };
    }
    /**
     * Sends an HTTP request.
     */
    function request(method: string, url: string, body?: string | null, headers?: { [key: string]: string }): Response;
    /**
     * Alias for request.
     */
    function fetch(method: string, url: string, body?: string | null, headers?: { [key: string]: string }): Response;
  }

  /**
   * Timer functions.
   */
  namespace timer {
    /**
     * Suspends execution for the given milliseconds.
     */
    function sleep(ms: number): Promise<null>;
  }

  /**
   * Date and time handling.
   */
  interface DateInstance {
    /**
     * Formats the date using strftime-style format string.
     * Default: "%Y-%m-%d %H:%M:%S"
     */
    format(fmt?: string): string;
    /**
     * Returns the timestamp in milliseconds.
     */
    timestamp(): number | string;
  }

  interface DateModule {
    /**
     * Creates a new Date instance.
     */
    "new"(val?: string | number): DateInstance;
    /**
     * Returns the current timestamp in milliseconds.
     */
    now(): number;
  }

  /**
   * Date and time handling namespace/module object.
   */
  const date: DateModule;

  /**
   * Process information.
   */
  namespace process {
    /**
     * Executes a shell command and returns its output.
     */
    function exec(command: string): { code: number, stdout: string, stderr: string };
  }

  /**
   * I/O operations namespace.
   */
  namespace io {
    /**
     * Prints arguments without a newline.
     */
    function print(...args: any[]): void;
    /**
     * Prints arguments followed by a newline.
     */
    function println(...args: any[]): void;
    /**
     * Reads a line from standard input.
     */
    function readline(): string;
  }

  /**
   * Set meta hooks for an object.
   */
  function setMeta(obj: object, meta: object): void;

  /**
   * Get meta hooks of an object.
   */
  function getMeta(obj: object): object | null;

  /**
   * Load a user-defined module.
   */
  function load(path: string): any;
}

/**
 * Standard console for I/O.
 */
declare namespace console {
  /**
   * Prints arguments followed by a newline.
   */
  function log(...args: any[]): void;
  /**
   * Alias for log.
   */
  function info(...args: any[]): void;
  /**
   * Alias for log.
   */
  function warn(...args: any[]): void;
  /**
   * Alias for log.
   */
  function error(...args: any[]): void;
  /**
   * Alias for log.
   */
  function debug(...args: any[]): void;
  /**
   * Prints arguments without a newline.
   */
  function print(...args: any[]): void;
  /**
   * Reads a line from standard input.
   * @deprecated non-standard, use Chen.io.readline if needed
   */
  function readLine(): string;
}

interface ChenIterator<T> {
  next(): { value: T | null, done: boolean };
}

/**
 * Object Constructor extension for Chen.
 */
interface ObjectConstructor {
  /**
   * Returns an iterator over the object keys.
   */
  iter(obj: object): ChenIterator<string>;
}

// Built-in prototypes extension
interface Array<T> {
    /**
   * The length of the array.
   */
  readonly length: number;
  /**
   * Returns an iterator for the array.
   */
  iter(): ChenIterator<T>;
  /**
   * Adds an element to the end of the array.
   */
  push(val: T): number;
  /**
   * Removes and returns the last element of the array.
   */
  pop(): T | null;
}

interface String {
  /**
   * The length of the string.
   */
  readonly length: number;
  /**
   * Removes whitespace from both ends of the string.
   */
  trim(): string;
  /**
   * Converts all characters to uppercase.
   */
  upper(): string;
  /**
   * Converts all characters to uppercase.
   */
  toUpperCase(): string;
  /**
   * Converts all characters to lowercase.
   */
  lower(): string;
  /**
   * Converts all characters to lowercase.
   */
  toLowerCase(): string;
  /**
   * Returns an iterator for the string characters.
   */
  iter(): ChenIterator<string>;
}
