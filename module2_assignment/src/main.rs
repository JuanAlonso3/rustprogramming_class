fn sum_with_step(total: &mut i32, low: i32, high: i32, step: i32) {
    
    let mut _i = low;
    while high >= _i{
        *total += _i;
        _i += step;
    }
  
}


fn most_frequent_word(text: &str) -> (String, usize) {
    let mut word_list: Vec<(String, usize)> = Vec::new();
    let mut neword = String::new();

    let mut copy = text.to_string();
    copy.push('*'); 
    for c in copy.chars() {
        if c != ' ' {
            neword.push(c);
        } 
        else if c == ' '  || c == '*'{
            if !neword.is_empty() {
                let mut found = false;
                for (word, count) in word_list.iter_mut() {
                    if *word == neword {
                        *count += 1;
                        found = true;
                        break;
                    }
                }
                if !found {
                    word_list.push((neword.clone(), 1));
                }
                neword.clear();
            }
        }
    }

    if !neword.is_empty() {
        let mut found = false;
        for (word, count) in word_list.iter_mut() {
            if *word == neword {
                *count += 1;
                found = true;
                break;
            }
        }
        if !found {
            word_list.push((neword.clone(), 1));
        }
    }

    let mut tup = (String::new(), 0);
    for (word, count) in &word_list {
        if *count > tup.1 {
            tup.0 = word.clone();
            tup.1 = *count;
        }
    }

    tup
}

fn main() {
    let mut result = 0;
    sum_with_step(&mut result, 0, 100, 1);
    println!("Sum 0 to 100, step 1: {}", result);

    result = 0;
    sum_with_step(&mut result, 0, 10, 2);
    println!("Sum 0 to 10, step 2: {}", result);

    result = 0;
    sum_with_step(&mut result, 5, 15, 3);
    println!("Sum 5 to 15, step 3: {}", result);

    
    let text = "the quick brown fox jumps over the lazy dog the quick brown fox";
    let (word, count) = most_frequent_word(text);
    println!("Most frequent word: \"{}\" ({} times)", word, count);
    
}

