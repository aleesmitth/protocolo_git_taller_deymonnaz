pub const GIT: &str = ".git";
pub const OBJECT: &str = ".git/objects";
pub const PACK: &str = ".git/pack";
pub const PARENT: &str = "parent";

pub const TREE_FILE_MODE: &str = "100644";
pub const TREE_SUBTREE_MODE: &str = "040000";
pub const DELETE_FLAG: &str = "-d";
pub const RENAME_FLAG: &str = "-m";
pub const TYPE_FLAG: &str = "-t";
pub const WRITE_FLAG: &str = "-w";
pub const SIZE_FLAG: &str = "-s";
pub const MESSAGE_FLAG: &str = "-m";
pub const VERIFY_FLAG: &str = "-v";
pub const LIST_FLAG: &str = "-l";
pub const CONTINUE_FLAG: &str = "--continue";

// flags for ls-files. also DELETE_FLAG is being used
pub const CACHED_FLAG: &str = "-c";
pub const IGNORE_FLAG: &str = "-i";
pub const STAGE_FLAG: &str = "-s";
pub const MODIFIED_FLAG: &str = "-m";

// flags for ls-tree
pub const DIRECT_FLAG: &str = "-d";
pub const RECURSE_FLAG: &str = "-r";
pub const LONG_FLAG: &str = "-l";

pub const EXCLUDE_LOG_ENTRY: char = '^';
pub const HEAD: &str = "HEAD";
pub const REBASE_HEAD: &str = ".git/REBASE_HEAD";
pub const ADD_FLAG: &str = "add";
pub const REMOVE_FLAG: &str = "rm";
pub const R_HEADS: &str = ".git/refs/heads";
pub const HEAD_FILE: &str = ".git/HEAD";
pub const R_TAGS: &str = ".git/refs/tags";
pub const R_REMOTES: &str = ".git/refs/remotes";
pub const DEFAULT_BRANCH_NAME: &str = "master";
pub const INDEX_FILE: &str = ".git/index";
pub const CONFIG_FILE: &str = ".git/config";
//pub const RELATIVE_PATH: &str = "RELATIVE_PATH";

pub const SERVER_BASE_PATH: &str = "src/server/";
pub const DEFAULT_REMOTE_REPOSITORY: &str = "origin";
pub const RECEIVED_PACK_FILE: &str = ".git/pack/received_pack_file.pack";

// flags for UnpackObjects
pub const VARINT_ENCODING_BITS: u8 = 7;
pub const VARINT_CONTINUE_FLAG: u8 = 1 << VARINT_ENCODING_BITS;
pub const TYPE_BITS: u8 = 3;
pub const TYPE_BYTE_SIZE_BITS: u8 = VARINT_ENCODING_BITS - TYPE_BITS;
pub const COPY_INSTRUCTION_FLAG: u8 = 1 << 7;
pub const COPY_OFFSET_BYTES: u8 = 4;
pub const COPY_SIZE_BYTES: u8 = 3;
pub const COPY_ZERO_SIZE: usize = 0x10000;

//CODES FOR COLORS IN TEXT
pub const COLOR_GREEN_CODE: &str = "\x1b[32m";
pub const COLOR_YELLOW_CODE: &str = "\x1b[33m";
pub const COLOR_RED_CODE: &str = "\x1b[31m";
pub const COLOR_RESET_CODE: &str = "\x1b[0m";

pub const DEFAULT_HEAD_LINE: &str = "ref: refs/heads/";

// consts used for server/client protocol
pub const ZERO_HASH: &str = "0000000000000000000000000000000000000000";
pub const LENGTH_BYTES: usize = 4;
pub const PULL_REQUEST_FILE: &str = "pull_requests.txt";
pub const REQUEST_DELIMITER_DONE: &str = "done\n";
pub const REQUEST_LENGTH_CERO: &str = "0000";
pub const SEPARATOR_PULL_REQUEST_FILE: &str = "\n";
pub const NAK_RESPONSE: &str = "NAK\n";
pub const WANT_REQUEST: &str = "want";
pub const UNPACK_CONFIRMATION: &str = "unpack ok\n";
pub const ALL_BRANCHES_LOCK: &str = "all_branches_lock";

// consts for marking conflicts in file
pub const CONFLICT_START: &str = "<<<<<<< HEAD";
pub const CONFLICT_BRANCH_CHANGE: &str = "=======";
pub const CONFLICT_END: &str = ">>>>>>>";
pub const MERGE_HEAD: &str = ".git/MERGE_HEAD";