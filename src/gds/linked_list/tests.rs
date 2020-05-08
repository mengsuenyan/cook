#[test]
fn push_pop() {
    let mut l = super::LinkedList::new();
    let mut l2 = super::LinkedList::new();
    for i in 0..200 {
        l.push_back(i);
        l2.push_front(i);
    }
    
    assert_eq!(l, l);
    let mut l_itr = l.iter();
    let mut l1_itr = l2.iter().rev();
    for _ in 0..l.len() {
        assert_eq!(l_itr.next(), l1_itr.next());
    }
    
    for _ in 0..32 {
        l.pop_back();
    }
    assert_eq!(l.len(), 200 - 32);
    
    assert_eq!(l2.find_by_val(&100), Some(99));
    assert_eq!(*l2.find_by_idx(99).unwrap(), 100);
    
    for _ in 0..68 {
        l.pop_front();
    }

    let mut l2 = super::LinkedList::new();
    for i in 68..168 {
        l2.push_back(i);
    }
    assert_eq!(l,l2);
}