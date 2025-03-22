#[derive(Debug)]
pub enum FilterType {
    INBOX,
    TODAY,
    SCHEDULED,
    PINBOARD,
    LABELS,
    COMPLETED,
}

impl FilterType {
    pub(crate) fn to_string(&self) -> String {
        match self {
            FilterType::INBOX => "Inbox".to_string(),
            FilterType::TODAY => "Today".to_string(),
            FilterType::SCHEDULED => "Scheduled".to_string(),
            FilterType::PINBOARD => "Pinboard".to_string(),
            FilterType::LABELS => "Labels".to_string(),
            FilterType::COMPLETED => "Completed".to_string(),
        }
    }

    pub(crate) fn get_icon(&self) -> String {
        match self {
            FilterType::INBOX => "mailbox-symbolic".to_string(),
            FilterType::TODAY => "star-outline-thick-symbolic".to_string(),
            FilterType::SCHEDULED => "month-symbolic".to_string(),
            FilterType::PINBOARD => "pin-symbolic".to_string(),
            FilterType::LABELS => "tag-outline-symbolic".to_string(),
            FilterType::COMPLETED => "check-round-outline-symbolic".to_string(),
        }
    }
    pub(crate) fn get_color(&self) -> String {
        match self {
            FilterType::INBOX => "#3584e4".to_string(),
            FilterType::TODAY => "#33d17a".to_string(),
            FilterType::SCHEDULED => "#9141ac".to_string(),
            FilterType::PINBOARD => "#ed333b".to_string(),
            FilterType::LABELS => "#986a44".to_string(),
            FilterType::COMPLETED => "#ff7800".to_string(),
        }
    }
}
