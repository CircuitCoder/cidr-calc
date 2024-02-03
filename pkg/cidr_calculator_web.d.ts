/* tslint:disable */
/* eslint-disable */
/**
* @returns {EvalState}
*/
export function create_state(): EvalState;
/**
* @param {EvalState} state
* @returns {(string)[]}
*/
export function print_scope(state: EvalState): (string)[];
/**
* @param {EvalState} state
* @param {string} input
* @returns {(string)[]}
*/
export function eval_input(state: EvalState, input: string): (string)[];
/**
*/
export class EvalState {
  free(): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_evalstate_free: (a: number) => void;
  readonly create_state: () => number;
  readonly print_scope: (a: number, b: number) => void;
  readonly eval_input: (a: number, b: number, c: number, d: number) => void;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {SyncInitInput} module
*
* @returns {InitOutput}
*/
export function initSync(module: SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
