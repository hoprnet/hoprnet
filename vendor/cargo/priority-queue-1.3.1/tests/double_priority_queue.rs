/*
 *  Copyright 2017 Gianmarco Garrisi and contributors
 *
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU Lesser General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version, or (at your opinion) under the terms
 *  of the Mozilla Public License version 2.0.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU Lesser General Public License for more details.
 *
 *  You should have received a copy of the GNU Lesser General Public License
 *  along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 */

#[cfg(test)]
mod doublepq_tests {
    pub use priority_queue::DoublePriorityQueue;

    #[test]
    fn init() {
        let pq: DoublePriorityQueue<String, i32> = DoublePriorityQueue::new();
        println!("{:?}", pq);
    }

    #[test]
    fn push_len() {
        let mut pq = DoublePriorityQueue::new();
        pq.push("a", 1);
        pq.push("b", 2);
        println!("{:?}", pq);
        assert_eq!(pq.len(), 2);
    }

    #[test]
    fn push_pop() {
        let mut pq = DoublePriorityQueue::new();
        assert_eq!(pq.peek_max(), None);
        assert_eq!(pq.peek_min(), None);
        assert_eq!(pq.pop_min(), None);
        assert_eq!(pq.pop_max(), None);
        pq.push("a", 1);
        pq.push("b", 2);
        pq.push("f", 7);
        pq.push("g", 4);
        pq.push("h", 3);
        println!("{:?}", pq);
        assert_eq!(pq.pop_max(), Some(("f", 7)));
        println!("{:?}", pq);
        assert_eq!(pq.peek_max(), Some((&"g", &4)));
        assert_eq!(pq.peek_min(), Some((&"a", &1)));
        assert_eq!(pq.pop_max(), Some(("g", 4)));
        assert_eq!(pq.len(), 3);
    }

    #[test]
    fn push_update() {
        let mut pq = DoublePriorityQueue::new();
        pq.push("a", 9);
        pq.push("b", 8);
        pq.push("c", 7);
        pq.push("d", 6);
        pq.push("e", 5);
        pq.push("f", 4);
        pq.push("g", 10);
        pq.push("k", 11);

        assert_eq!(pq.push("d", 20), Some(6));
        assert_eq!(pq.peek_max(), Some((&"d", &20)));
        assert_eq!(pq.pop_max(), Some(("d", 20)));
    }

    #[test]
    fn push_increase() {
        let mut pq = DoublePriorityQueue::new();
        pq.push("Processor", 1);
        pq.push("Mainboard", 2);
        pq.push("RAM", 5);
        pq.push("GPU", 4);
        pq.push("Disk", 3);

        let processor_priority = |pq: &DoublePriorityQueue<&str, i32>| {
            *pq.iter()
                .filter_map(|(i, p)| if *i == "Processor" { Some(p) } else { None })
                .next()
                .unwrap()
        };

        pq.push_increase("SSD", 5);
        assert_eq!(pq.get("SSD"), Some((&"SSD", &5)));

        pq.push_increase("Processor", 3);
        assert_eq!(processor_priority(&pq), 3);

        pq.push_increase("Processor", 1);
        assert_eq!(processor_priority(&pq), 3);

        pq.push_increase("Processor", 6);
        assert_eq!(pq.peek_max(), Some((&"Processor", &6)));
    }

    #[test]
    fn change_priority1() {
        let mut pq = DoublePriorityQueue::new();
        assert_eq!(pq.push("Processor", 1), None);
        assert_eq!(pq.push("Mainboard", 2), None);
        assert_eq!(pq.push("RAM", 5), None);
        assert_eq!(pq.push("GPU", 4), None);
        assert_eq!(pq.push("Disk", 3), None);

        assert_eq!(pq.change_priority("SSD", 12), None);

        assert_eq!(pq.change_priority("Processor", 10), Some(1));
        assert_eq!(pq.peek_max(), Some((&"Processor", &10)));

        assert_eq!(pq.change_priority("RAM", 11), Some(5));
        assert_eq!(pq.peek_max(), Some((&"RAM", &11)));
    }

    #[test]
    fn change_priority_does_not_change_contents() {
        use std::hash::{Hash, Hasher};
        struct MyFn {
            name: &'static str,
            func: fn(&mut i32),
        }
        impl Default for MyFn {
            fn default() -> Self {
                Self {
                    name: "",
                    func: |_| {},
                }
            }
        }
        impl PartialEq for MyFn {
            fn eq(&self, other: &Self) -> bool {
                self.name == other.name
            }
        }
        impl Eq for MyFn {}
        impl Hash for MyFn {
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.name.hash(state);
            }
        }
        impl std::fmt::Debug for MyFn {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write![f, "{:?}", self.name]
            }
        }

        let mut pq = DoublePriorityQueue::new();
        pq.push(
            MyFn {
                name: "increment-one",
                func: |x| *x += 1,
            },
            2,
        );
        pq.push(
            MyFn {
                name: "increment-two",
                func: |x| *x += 2,
            },
            1,
        );

        let mut cnt = 0;
        assert_eq![
            pq.peek_max(),
            Some((
                &MyFn {
                    name: "increment-one",
                    func: |_| {}
                },
                &2
            ))
        ];
        pq.change_priority(
            &MyFn {
                name: "increment-one",
                func: |_| {},
            },
            0,
        );
        assert_eq![
            pq.peek_max(),
            Some((
                &MyFn {
                    name: "increment-two",
                    func: |_| {}
                },
                &1
            ))
        ];

        assert_eq![cnt, 0];

        (pq.pop_max().unwrap().0.func)(&mut cnt);
        assert_eq![cnt, 2];

        (pq.pop_max().unwrap().0.func)(&mut cnt);
        assert_eq![cnt, 3];
    }

    #[test]
    fn reversed_order() {
        use std::cmp::Reverse;
        let mut pq: DoublePriorityQueue<_, Reverse<i32>> = DoublePriorityQueue::new();
        pq.push("a", Reverse(1));
        pq.push("b", Reverse(2));
        assert_eq![pq.pop_max(), Some(("a", Reverse(1)))];
    }

    #[test]
    fn from_vec() {
        let v = vec![("a", 1), ("b", 2), ("f", 7)];
        let mut pq: DoublePriorityQueue<_, _> = DoublePriorityQueue::from(v);
        assert_eq!(pq.pop_max(), Some(("f", 7)));
        assert_eq!(pq.len(), 2);
    }

    #[test]
    fn from_vec_with_repeated() {
        let v = vec![("a", 1), ("b", 2), ("f", 7), ("a", 2)];
        let mut pq: DoublePriorityQueue<_, _> = v.into();
        assert_eq!(pq.pop_max(), Some(("f", 7)));
        assert_eq!(pq.len(), 2);
    }

    #[test]
    fn from_iter() {
        use std::iter::FromIterator;

        let v = vec![("a", 1), ("b", 2), ("f", 7)];
        let mut pq: DoublePriorityQueue<_, _> = DoublePriorityQueue::from_iter(v.into_iter());
        assert_eq!(pq.pop_max(), Some(("f", 7)));
        assert_eq!(pq.len(), 2);
    }

    #[test]
    fn heap_sort() {
        type Pq<I, P> = DoublePriorityQueue<I, P>;

        let v = vec![("a", 2), ("b", 7), ("f", 1)];
        let sorted = (Pq::from(v)).into_descending_sorted_vec();
        assert_eq!(sorted.as_slice(), &["b", "a", "f"]);
    }

    #[test]
    fn change_priority_by() {
        use std::iter::FromIterator;

        let v = vec![("a", 1), ("b", 2), ("f", 7), ("g", 6), ("h", 5)];
        let mut pq: DoublePriorityQueue<_, _> = DoublePriorityQueue::from_iter(v.into_iter());

        assert!(!pq.change_priority_by("z", |z| *z += 8));

        assert!(pq.change_priority_by("b", |b| *b += 8));
        assert_eq!(
            pq.into_descending_sorted_vec().as_slice(),
            &["b", "f", "g", "h", "a"]
        );
    }

    #[test]
    fn remove_empty() {
        let mut pq: DoublePriorityQueue<&str, i32> = DoublePriorityQueue::new();

        assert_eq!(pq.remove(&"b"), None);
        assert_eq!(pq.len(), 0);
    }

    #[test]
    fn remove_one() {
        let mut pq = DoublePriorityQueue::new();

        assert_eq!(pq.push("b", 21), None);

        assert_eq!(Some(("b", 21)), pq.remove(&"b"));
        assert_eq!(pq.len(), 0);
    }

    #[test]
    fn remove() {
        use std::iter::FromIterator;
        type Pq<I, P> = DoublePriorityQueue<I, P>;

        let v = vec![("a", 1), ("b", 2), ("f", 7), ("g", 6), ("h", 5)];
        let mut pq = Pq::from_iter(v.into_iter());

        pq.remove(&"b").unwrap();
        assert!(pq.remove(&"b").is_none());
        pq.push("b", 2);
        pq.remove(&"b");
        assert_eq!(
            ["f", "g", "h", "a"],
            pq.into_descending_sorted_vec().as_slice()
        );
    }

    #[test]
    fn remove2() {
        use std::collections::hash_map::RandomState;
        let mut queue = DoublePriorityQueue::<i32, i32, RandomState>::default();

        for i in 0..7 {
            queue.push(i, i);
        }

        queue.remove(&0);

        let mut last_priority = *queue.peek_max().unwrap().1;
        while let Some((_, priority)) = queue.pop_max() {
            assert!(last_priority >= priority);
            last_priority = priority;
        }

        let mut queue: DoublePriorityQueue<i32, i32, RandomState> =
            [20, 7, 19, 5, 6, 15, 18, 1, 2, 3, 4, 13, 14, 16, 17]
                .iter()
                .map(|i| (*i, *i))
                .collect();

        queue.remove(&1);

        let mut last_priority = *queue.peek_max().unwrap().1;
        while let Some((_, priority)) = queue.pop_max() {
            assert!(last_priority >= priority);
            last_priority = priority;
        }
    }

    #[test]
    fn extend() {
        let mut pq = DoublePriorityQueue::new();
        pq.push("a", 1);
        pq.push("b", 2);
        pq.push("f", 7);

        let v = vec![("c", 4), ("d", 6), ("e", 3)];
        pq.extend(v);
        assert_eq!(pq.len(), 6);
        assert_eq!(
            pq.into_descending_sorted_vec().as_slice(),
            &["f", "d", "c", "e", "b", "a"]
        );
    }

    #[test]
    fn extend_empty() {
        let mut pq = DoublePriorityQueue::new();

        let v = vec![("c", 4), ("d", 6), ("e", 3)];
        pq.extend(v);
        assert_eq!(pq.len(), 3);
        assert_eq!(pq.into_descending_sorted_vec().as_slice(), &["d", "c", "e"]);
    }

    #[test]
    fn iter() {
        let mut pq = DoublePriorityQueue::new();
        pq.push("a", 1);
        pq.push("b", 2);
        pq.push("f", 7);

        assert_eq!(pq.iter().count(), 3);
    }

    #[test]
    fn iter_mut() {
        let mut pq = DoublePriorityQueue::new();
        pq.push("a", 1);
        pq.push("b", 2);
        pq.push("f", 7);
        pq.push("g", 4);
        pq.push("h", 3);

        for (i, p) in &mut pq {
            if *i < "f" {
                *p += 18;
            }
        }

        assert_eq!(pq.pop_max(), Some(("b", 20)));

        /*
        As expected, this does not compile
        let iter_mut = pq.iter_mut();
        iter_mut.for_each(|(_, p)| {*p += 2});

        assert_eq!(pq.pop_max(), Some(("f", 9)));
        */
    }

    #[test]
    fn into_sorted_iter() {
        let mut pq = DoublePriorityQueue::new();
        pq.push("a", 1);
        pq.push("b", 2);
        pq.push("f", 7);

        assert_eq!(
            pq.into_sorted_iter().collect::<Vec<_>>(),
            vec!(("a", 1), ("b", 2), ("f", 7))
        );
    }

    #[test]
    fn into_sorted_iter_reverse() {
        let mut pq = DoublePriorityQueue::new();
        pq.push("a", 1);
        pq.push("b", 2);
        pq.push("f", 7);

        assert_eq!(
            pq.into_sorted_iter().rev().collect::<Vec<_>>(),
            vec!(("f", 7), ("b", 2), ("a", 1))
        );
    }

    #[test]
    fn iter_mut1() {
        let mut queue: DoublePriorityQueue<&'static str, i32> = Default::default();

        queue.push("a", 0);
        queue.push("b", 1);
        assert_eq!(queue.peek_max().unwrap().0, &"b");

        let iter_mut = queue.iter_mut();
        for (k, v) in iter_mut {
            if k == &"a" {
                *v = 2;
            }
        }

        assert_eq!(queue.peek_max().unwrap().0, &"a");
    }

    #[test]
    fn iter_mut_empty() {
        let mut queue: DoublePriorityQueue<&'static str, i32> = Default::default();

        let iter_mut = queue.iter_mut();
        for (k, v) in iter_mut {
            if k == &"a" {
                *v = 2;
            }
        }
    }

    #[test]
    fn eq() {
        let mut a = DoublePriorityQueue::new();
        let mut b = DoublePriorityQueue::new();
        assert_eq!(a, b);

        a.push("a", 1);
        b.push("a", 1);
        assert_eq!(a, b);

        a.push("b", 2);
        assert_ne!(a, b);

        b.push("f", 7);
        b.push("g", 4);
        b.push("h", 3);
        b.push("b", 2);

        a.push("g", 4);
        a.push("f", 7);
        a.push("h", 3);
        assert_eq!(a, b);
        assert_eq!(b, a);
    }

    #[test]
    fn non_default_key() {
        use std::time::*;
        type PqType = DoublePriorityQueue<i32, Instant>;
        let _: PqType = DoublePriorityQueue::default();
    }

    #[test]
    fn conversion() {
        use priority_queue::PriorityQueue;

        let mut pq = PriorityQueue::new();

        pq.push('a', 3);
        pq.push('b', 5);
        pq.push('c', 1);

        let mut dpq: DoublePriorityQueue<_, _> = pq.into();

        assert_eq!(dpq.pop_max(), Some(('b', 5)));
        assert_eq!(dpq.pop_min(), Some(('c', 1)));
    }

    #[test]
    fn user_test() {
        use priority_queue::PriorityQueue;
        use std::cmp::Reverse;

        let mut double_queue = DoublePriorityQueue::new();
        let mut simple_queue = PriorityQueue::new();
        let data = vec![
            23403400, 23375200, 23395900, 23400100, 23401600, 23375200, 23391100, 23410300,
            23391600, 23402200, 23407600, 23414500, 23421000, 23430500, 23430500, 23441600,
            23441600, 23440200, 23424200, 23465500, 23273300, 23232300, 23183100, 23315600,
            23328500, 23328500, 23346900, 23350000, 23363600, 23388100, 23403600, 23385300,
            23385800, 23385800, 23363600, 23363600, 23398900, 23400700, 23401700, 23370000,
            23370000, 23396700, 23370000, 23370000, 23396800, 23385000, 23369100, 23381800,
            23372300, 23382600, 23382300, 23401500, 23405000, 23392000, 23392000, 23398400,
            23393800, 23382300, 23382800, 23376200, 23379300, 23380600, 23380600, 23384400,
            23379500, 23381700, 23391700, 23396300, 23402000, 23414800, 23410900, 23423600,
            23419300, 23419300, 23420900, 23397100, 23425900, 23397100, 23397100, 23417400,
            23413600, 23422700, 23422700, 23413600, 23407400, 23413900, 23407400, 23397100,
            23391100, 23390000, 23388500, 23383000, 23375900, 23355100, 23354300, 23370400,
            23381800, 23381800, 23383600, 23391400, 23393800, 23393800, 23379100, 23380800,
            23384900, 23384900, 23390200, 23382000, 23380800, 23379900, 23370500, 23377300,
            23378000, 23391700, 23386700, 23374200, 23392200, 23394500, 23403200, 23413500,
            23420400, 23443800, 23441600, 23441600, 23485300, 23488500, 23488500, 23505300,
            23471400, 23471400, 23477900, 23488700, 23488700, 23483200, 23463200, 23463200,
            23463600, 23463600, 23468800, 23485700, 23487800, 23508600, 23512900, 23507700,
            23507700, 23539200, 23541700, 23539200, 23507700, 23487800, 23487800, 23553500,
            23553500, 23553500, 23553500, 23553500, 23553500, 23565500, 23566700, 23566700,
            23566700, 23565500, 23566700, 23565500, 23566700, 23566700, 23549900, 23541300,
            23560800, 23566800, 23563500, 23563500, 23541300, 23538300, 23535300, 23543700,
            23549100, 23549600, 23520000, 23500000, 23545600, 23554300, 23558600, 23526700,
            23452100, 23526700, 23520000, 23520000, 23526700, 23524300, 23524300, 23526700,
            23524300, 23450000, 23450000, 23354300, 23350000, 23521100, 23542700, 23553700,
        ];
        let window = 32;
        for idx in 0..window {
            let val = data[idx];
            simple_queue.push(idx, Reverse(val));
            double_queue.push(idx, val);
        }
        for idx in window..data.len() {
            let val = data[idx];
            simple_queue.change_priority(&(idx % window), Reverse(val));
            double_queue.change_priority(&(idx % window), val);
            let simple_min_result = simple_queue.peek().unwrap().1.clone().0;
            let double_min_result = double_queue.peek_min().unwrap().1.clone();
            assert_eq!(
                simple_min_result,
                double_min_result,
                "{} {:?} {:?}\n {:?}\n{:?}",
                idx,
                simple_queue.peek(),
                double_queue.peek_min(),
                simple_queue,
                double_queue
            );
        }
    }
}

#[cfg(all(feature = "serde", test))]
mod serde_tests_basics {
    use priority_queue::DoublePriorityQueue;
    use serde_test::{assert_tokens, Token};
    #[test]
    fn serde_empty() {
        let pq: DoublePriorityQueue<String, i32> = DoublePriorityQueue::new();

        assert_tokens(&pq, &[Token::Seq { len: Some(0) }, Token::SeqEnd]);
    }

    #[test]
    fn serde() {
        let mut pq = DoublePriorityQueue::new();

        pq.push("a", 1);
        pq.push("b", 2);
        pq.push("f", 7);
        pq.push("g", 4);
        pq.push("h", 3);

        assert_tokens(
            &pq,
            &[
                Token::Seq { len: Some(5) },
                Token::Tuple { len: 2 },
                Token::BorrowedStr("a"),
                Token::I32(1),
                Token::TupleEnd,
                Token::Tuple { len: 2 },
                Token::BorrowedStr("b"),
                Token::I32(2),
                Token::TupleEnd,
                Token::Tuple { len: 2 },
                Token::BorrowedStr("f"),
                Token::I32(7),
                Token::TupleEnd,
                Token::Tuple { len: 2 },
                Token::BorrowedStr("g"),
                Token::I32(4),
                Token::TupleEnd,
                Token::Tuple { len: 2 },
                Token::BorrowedStr("h"),
                Token::I32(3),
                Token::TupleEnd,
                Token::SeqEnd,
            ],
        );
    }
}

//more complex tests
//thanks to ckaran
#[cfg(all(feature = "serde", test))]
mod serde_tests_custom_structs {
    use priority_queue::DoublePriorityQueue;
    use std::cmp::{Ord, Ordering, PartialOrd};
    use std::default::Default;
    use std::time::Duration;
    use uuid::Uuid;

    use serde::{Deserialize, Serialize};

    // Abusing Duration as a mutable std::time::Instant
    type ActivationDate = Duration;

    /// Events are compared by EventComparables instances.
    ///
    /// EventComparables instances are similar to instances of time, but with the
    /// extra wrinkle of having a Uuid instance.  When EventComparables instances
    /// are compared, they are first compared by their activation date, with the
    /// date that occurs earlier being greater than a date that occurs later. This
    /// ordering exists because of how priority_queue::DoublePriorityQueue works; it is
    /// naturally a max priority queue; using this ordering makes it a min
    /// priority queue. EventComparables go one step beyond using time as the key
    /// though; they  also have uuid::Uuid instances which are used to guarantee
    /// that every EventComparables is unique.  This ensures that if a set of
    /// events all  occur at the same time, they will still be executed in a
    /// deterministic order, every single time the queue's contents are executed.
    /// This is  critical for deterministic simulators.
    #[derive(Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
    #[serde(default)]
    #[serde(deny_unknown_fields)]
    struct EventComparables {
        /// This is when the event will be fired.
        activation_date: ActivationDate,

        /// This is a unique ID.  Except for ensuring that different events are
        /// guaranteed to compare as being different, it has no purpose.
        id: Uuid,
    }

    /// Default events activate at time (0, 0)
    ///
    /// All default events first at time (0, 0), but every single one has a unique
    /// id.  This ensures that regardless of the number of default events you
    /// create, they will always execute in the same order.
    impl Default for EventComparables {
        fn default() -> Self {
            EventComparables {
                activation_date: ActivationDate::new(0, 0),
                id: Uuid::new_v4(),
            }
        }
    }

    /// The priority queue depends on `Ord`. Explicitly implement the trait so the
    /// queue becomes a min-heap instead of a max-heap.
    impl Ord for EventComparables {
        fn cmp(&self, other: &Self) -> Ordering {
            // Notice that the we flip the ordering on costs. In case of a tie we
            // compare by id - this step is necessary to make implementations of
            // `PartialEq` and `Ord` consistent.

            other
                .activation_date
                .cmp(&self.activation_date)
                .then_with(|| self.id.cmp(&other.id))
        }
    }

    // `PartialOrd` needs to be implemented as well.
    impl PartialOrd for EventComparables {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    /// A fake event to fire on some date.
    ///
    /// This is a fake event that I'll fire when the corresponding
    /// EventComparables instance comes up.  The contents are immaterial; I'm just
    /// using it for testing
    #[derive(Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize, Debug)]
    #[serde(default)]
    #[serde(deny_unknown_fields)]
    struct ConcreteEvent1 {
        a: i32,
        b: i64,
    }

    impl Default for ConcreteEvent1 {
        fn default() -> Self {
            ConcreteEvent1 { a: 0, b: 0 }
        }
    }

    //////////////////////////////////////////////////////////////////////////////
    // Test 1
    //////////////////////////////////////////////////////////////////////////////
    #[test]
    fn test1() {
        println!("test1()");

        type PqType = DoublePriorityQueue<i32, i32>;

        let mut pq: PqType = DoublePriorityQueue::new();
        pq.push(0, 0);
        pq.push(1, 1);

        let serialized = serde_json::to_string(&pq).unwrap();
        println!("serialized = {:?}", serialized);
        let deserialized: PqType = serde_json::from_str(&serialized).unwrap();
        println!("deserialized = {:?}", deserialized);
    }

    //////////////////////////////////////////////////////////////////////////////
    // Test 2
    //////////////////////////////////////////////////////////////////////////////
    #[test]
    fn test2() {
        println!("\n\ntest2()");

        type PqType = DoublePriorityQueue<i32, EventComparables>;

        let mut pq: PqType = DoublePriorityQueue::new();
        pq.push(0, Default::default()); // Uuids will be different
        pq.push(1, Default::default());

        let serialized = serde_json::to_string(&pq).unwrap();
        println!("serialized = {:?}", serialized);
        let deserialized: PqType = serde_json::from_str(&serialized).unwrap();
        println!("deserialized = {:?}", deserialized);
    }

    //////////////////////////////////////////////////////////////////////////////
    // Test 3
    //////////////////////////////////////////////////////////////////////////////
    #[test]
    fn test3() {
        println!("\n\ntest3()");

        // Create some concrete events and comparables, and test to see that they
        // can be serialized/deserialized

        let ce1 = ConcreteEvent1 { a: 12, b: 45 };
        let ec1 = EventComparables {
            activation_date: ActivationDate::new(0, 1),
            id: Uuid::new_v4(),
        };

        let ce2 = ConcreteEvent1 { a: 37, b: 123 };
        let ec2 = EventComparables {
            activation_date: ActivationDate::new(0, 1),
            id: Uuid::new_v4(),
        };

        let serialized = serde_json::to_string(&ce1).unwrap();
        println!("serialized = {:?}", serialized);
        let deserialized: ConcreteEvent1 = serde_json::from_str(&serialized).unwrap();
        println!("deserialized = {:?}", deserialized);
        assert_eq!(ce1, deserialized);

        let serialized = serde_json::to_string(&ec1).unwrap();
        println!("serialized = {:?}", serialized);
        let deserialized: EventComparables = serde_json::from_str(&serialized).unwrap();
        println!("deserialized = {:?}", deserialized);
        assert_eq!(ec1, deserialized);

        let serialized = serde_json::to_string(&ce2).unwrap();
        println!("serialized = {:?}", serialized);
        let deserialized: ConcreteEvent1 = serde_json::from_str(&serialized).unwrap();
        println!("deserialized = {:?}", deserialized);
        assert_eq!(ce2, deserialized);

        let serialized = serde_json::to_string(&ec2).unwrap();
        println!("serialized = {:?}", serialized);
        let deserialized: EventComparables = serde_json::from_str(&serialized).unwrap();
        println!("deserialized = {:?}", deserialized);
        assert_eq!(ec2, deserialized);

        {
            type PqType = DoublePriorityQueue<ConcreteEvent1, EventComparables>;

            let mut pq: PqType = DoublePriorityQueue::new();
            pq.push(ce1, ec1);
            pq.push(ce2, ec2);

            let serialized = serde_json::to_string(&pq).unwrap();
            println!("serialized = {:?}", serialized);
            let deserialized: PqType = serde_json::from_str(&serialized).unwrap();
            println!("deserialized = {:?}", deserialized);
        }
    }
}
