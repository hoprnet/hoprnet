#![no_std]

//! The `array-vec` crate allows you to initialize arrays
//! with an initializer closure that will be called
//! once for each element until the array is filled.
//!
//! This way you do not need to default-fill an array
//! before running initializers. Rust currently only
//! lets you either specify all initializers at once,
//! individually (`[a(), b(), c(), ...]`), or specify
//! one initializer for a `Copy` type (`[a(); N]`),
//! which will be called once with the result copied over.
//!
//! # Examples:
//! ```rust
//! # #![allow(unused)]
//! # extern crate array_init;
//!
//! // Initialize an array of length 10 containing
//! // successive squares
//!
//! let arr: [u32; 50] = array_init::array_init(|i| (i*i) as u32);
//!
//! // Initialize an array from an iterator
//! // producing an array of [1,2,3,4] repeated
//!
//! let four = [1u32,2,3,4];
//! let mut iter = four.iter().cloned().cycle();
//! let arr: [u32; 50] = array_init::from_iter(iter).unwrap();
//!
//! // Closures can also mutate state. We guarantee that they will be called
//! // in order from lower to higher indices.
//!
//! let mut last = 1u64;
//! let mut secondlast = 0;
//! let fibonacci: [u64; 50] = array_init::array_init(|_| {
//!     let this = last + secondlast;
//!     secondlast = last;
//!     last = this;
//!     this
//! });
//! ```
//!
//! Currently, using `from_iter` and `array_init` will incur additional
//! memcpys, which may be undesirable for a large array. This can be eliminated
//! by using the nightly feature of this crate, which uses unions to provide
//! panic-safety. Alternatively, if your array only contains `Copy` types,
//! you can use `array_init_copy` and `from_iter_copy`.
//!
//! Sadly, cannot guarantee right now that any of these solutions will completely
//! eliminate a memcpy.
//!

extern crate nodrop;

use nodrop::NoDrop;
use core::mem;

/// Trait for things which are actually arrays
///
/// Probably shouldn't implement this yourself,
/// but you can
pub unsafe trait IsArray {
    type Item;
    /// Must assume self is uninitialized.
    fn set(&mut self, idx: usize, value: Self::Item);
    fn len() -> usize;
}

#[inline]
/// Initialize an array given an initializer expression
///
/// The initializer is given the index of the element. It is allowed
/// to mutate external state; we will always initialize the elements in order.
///
/// Without the nightly feature it is very likely that this will cause memcpys.
/// For panic safety, we internally use NoDrop, which will ensure that panics
/// in the initializer will not cause the array to be prematurely dropped.
/// If you are using a Copy type, prefer using `array_init_copy` since
/// it does not need the panic safety stuff and is more likely to have no
/// memcpys.
///
/// If your initializer panics, any elements that have been initialized
/// will be leaked.
///
/// # Examples
///
/// ```rust
/// # #![allow(unused)]
/// # extern crate array_init;
///
/// // Initialize an array of length 10 containing
/// // successive squares
///
/// let arr: [u32; 50] = array_init::array_init(|i| (i*i) as u32);
///
/// // Initialize an array from an iterator
/// // producing an array of [1,2,3,4] repeated
///
/// let four = [1u32,2,3,4];
/// let mut iter = four.iter().cloned().cycle();
/// let arr: [u32; 50] = array_init::from_iter(iter).unwrap();
///
/// ```
///
pub fn array_init<Array, F>(mut initializer: F) -> Array where Array: IsArray,
                                                               F: FnMut(usize) -> Array::Item {
    // NoDrop makes this panic-safe
    // We are sure to initialize the whole array here,
    // and we do not read from the array till then, so this is safe.
    let mut ret: NoDrop<Array> = NoDrop::new(unsafe { mem::uninitialized() });
    for i in 0..Array::len() {
        Array::set(&mut ret, i, initializer(i));
    }
    ret.into_inner()
}

#[inline]
/// Initialize an array given an iterator
///
/// We will iterate until the array is full or the iterator is exhausted. Returns
/// None if the iterator is exhausted before we can fill the array.
///
/// Without the nightly feature it is very likely that this will cause memcpys.
/// For panic safety, we internally use NoDrop, which will ensure that panics
/// in the initializer will not cause the array to be prematurely dropped.
/// If you are using a Copy type, prefer using `from_iter_copy` since
/// it does not need the panic safety stuff and is more likely to have no
/// memcpys.
///
/// # Examples
///
/// ```rust
/// # #![allow(unused)]
/// # extern crate array_init;
///
/// // Initialize an array from an iterator
/// // producing an array of [1,2,3,4] repeated
///
/// let four = [1u32,2,3,4];
/// let mut iter = four.iter().cloned().cycle();
/// let arr: [u32; 50] = array_init::from_iter_copy(iter).unwrap();
/// ```
///
pub fn from_iter<Array, I>(iter: I) -> Option<Array>
    where I: IntoIterator<Item = Array::Item>,
          Array: IsArray {
    // NoDrop makes this panic-safe
    // We are sure to initialize the whole array here,
    // and we do not read from the array till then, so this is safe.
    let mut ret: NoDrop<Array> = NoDrop::new(unsafe { mem::uninitialized() });
    let mut count = 0;
    for item in iter.into_iter().take(Array::len()) {
        Array::set(&mut ret, count, item);
        count += 1;
    }
    // crucial for safety!
    if count == Array::len() {
        Some(ret.into_inner())
    } else {
        None
    }
}

#[inline]
/// Initialize an array of `Copy` elements given an initializer expression
///
/// The initializer is given the index of the element. It is allowed
/// to mutate external state; we will always initialize the elements in order.
///
/// This is preferred over `array_init` if you have a `Copy` type
///
/// # Examples
///
/// ```rust
/// # #![allow(unused)]
/// # extern crate array_init;
///
/// // Initialize an array of length 10 containing
/// // successive squares
///
/// let arr: [u32; 50] = array_init::array_init_copy(|i| (i*i) as u32);
///
///
/// // Closures can also mutate state. We guarantee that they will be called
/// // in order from lower to higher indices.
///
/// let mut last = 1u64;
/// let mut secondlast = 0;
/// let fibonacci: [u64; 50] = array_init::array_init_copy(|_| {
///     let this = last + secondlast;
///     secondlast = last;
///     last = this;
///     this
/// });
/// ```
///
pub fn array_init_copy<Array, F>(mut initializer: F) -> Array where Array: IsArray,
                                                                    F: FnMut(usize) -> Array::Item,
                                                                    Array::Item : Copy {
    // We are sure to initialize the whole array here,
    // and we do not read from the array till then, so this is safe.
    let mut ret: Array = unsafe { mem::uninitialized() };
    for i in 0..Array::len() {
        Array::set(&mut ret, i, initializer(i));
    }
    ret
}

#[inline]
/// Initialize an array given an iterator
///
/// We will iterate until the array is full or the iterator is exhausted. Returns
/// None if the iterator is exhausted before we can fill the array.
///
/// This is preferred over `from_iter_copy` if you have a `Copy` type
///
/// # Examples
///
/// ```rust
/// # #![allow(unused)]
/// # extern crate array_init;
///
/// // Initialize an array from an iterator
/// // producing an array of [1,2,3,4] repeated
///
/// let four = [1u32,2,3,4];
/// let mut iter = four.iter().cloned().cycle();
/// let arr: [u32; 50] = array_init::from_iter_copy(iter).unwrap();
/// ```
pub fn from_iter_copy<Array, I>(iter: I) -> Option<Array>
    where I: IntoIterator<Item = Array::Item>,
          Array: IsArray,
          Array::Item : Copy {
    // We are sure to initialize the whole array here,
    // and we do not read from the array till then, so this is safe.
    let mut ret: Array = unsafe { mem::uninitialized() };
    let mut count = 0;
    for item in iter.into_iter().take(Array::len()) {
        Array::set(&mut ret, count, item);
        count += 1;
    }
    // crucial for safety!
    if count == Array::len() {
        Some(ret)
    } else {
        None
    }
}

macro_rules! impl_is_array {
    ($($size:expr)+) => ($(
        unsafe impl<T> IsArray for [T; $size] {
            type Item = T;
            #[inline]
            fn set(&mut self, idx: usize, value: Self::Item) {
                mem::forget(mem::replace(&mut self[idx], value));
            }

            #[inline]
            fn len() -> usize {
                $size
            }
        }
    )+)
}

// lol

impl_is_array! {
     0  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15
    16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31
    32 33 34 35 36 37 38 39 40 41 42 43 44 45 46 47
    48 49 50 51 52 53 54 55 56 57 58 59 60 61 62 63
    64 65 66 67 68 69 70 71 72 73 74 75 76 77 78 79
    80 81 82 83 84 85 86 87 88 89 90 91 92 93 94 95
    96 97 98 99 100 101 102 103 104 105 106 107 108
    109 110 111 112 113 114 115 116 117 118 119 120
    121 122 123 124 125 126 127 128 129 130 131 132
    133 134 135 136 137 138 139 140 141 142 143 144
    145 146 147 148 149 150 151 152 153 154 155 156
    157 158 159 160 161 162 163 164 165 166 167 168
    169 170 171 172 173 174 175 176 177 178 179 180
    181 182 183 184 185 186 187 188 189 190 191 192
    193 194 195 196 197 198 199 200 201 202 203 204
    205 206 207 208 209 210 211 212 213 214 215 216
    217 218 219 220 221 222 223 224 225 226 227 228
    229 230 231 232 233 234 235 236 237 238 239 240
    241 242 243 244 245 246 247 248 249 250 251 252
    253 254 255 256 257 258 259 260 261 262 263 264
    265 266 267 268 269 270 271 272 273 274 275 276
    277 278 279 280 281 282 283 284 285 286 287 288
    289 290 291 292 293 294 295 296 297 298 299 300
    301 302 303 304 305 306 307 308 309 310 311 312
    313 314 315 316 317 318 319 320 321 322 323 324
    325 326 327 328 329 330 331 332 333 334 335 336
    337 338 339 340 341 342 343 344 345 346 347 348
    349 350 351 352 353 354 355 356 357 358 359 360
    361 362 363 364 365 366 367 368 369 370 371 372
    373 374 375 376 377 378 379 380 381 382 383 384
    385 386 387 388 389 390 391 392 393 394 395 396
    397 398 399 400 401 402 403 404 405 406 407 408
    409 410 411 412 413 414 415 416 417 418 419 420
    421 422 423 424 425 426 427 428 429 430 431 432
    433 434 435 436 437 438 439 440 441 442 443 444
    445 446 447 448 449 450 451 452 453 454 455 456
    457 458 459 460 461 462 463 464 465 466 467 468
    469 470 471 472 473 474 475 476 477 478 479 480
    481 482 483 484 485 486 487 488 489 490 491 492
    493 494 495 496 497 498 499 500 501 502 503 504
    505 506 507 508 509 510 511 512
}
