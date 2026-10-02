/** Compact hero / model poster — relationships first. */
export const SAMPLE_HERO = `namespace app

table User {
  id: uuid
  profile: Profile
  sessions: ref<Session>[]
}`;

/** Persistence shapes (`table`, keys, references). */
export const SAMPLE_PERSISTENCE = `namespace demo::identity;

enums AccessVisibility {
    [unknown]
    Unknown = 0,
    Public = 1,
    Internal = 2,
    Private = 3,
}

flags ApiKeyScope {
    [unknown]
    Unknown = 0,
    ReadProfile = 0x01,
    ReadContent = 0x02,
    Generate = 0x04,
}

table User {
    @@user_id: uuid,
    @slug: utf8,
    password_hash: utf8,
    display_name: utf8,
    bio: utf8?,
    manager: &User? = null,
    created_at: DateTime<UTC>,
}

table LoginEmail {
    @@login_email_id: uuid,
    @email: utf8,
    user_id: &User,
    verified_at: DateTime<UTC>?,
}`;

/** Communication shapes (`class`, `service`, routes). */
export const SAMPLE_SERVICE = `namespace demo::identity;
using demo::billing::VipTier;

class UserProfile {
    user_id: uuid,
    slug: utf8,
    email: utf8,
    display_name: utf8,
    bio: utf8?,
    vip_tier: VipTier,
}

class LoginRequest {
    email: utf8,
    password: utf8,
}

class LoginResponse {
    access_token: utf8,
    user: UserProfile,
}

[tag(LoginKind)]
union LoginResult(i32) {
    [tag(1)] Authenticated { response: LoginResponse, },
    [tag(2)] InvalidCredentials { message: utf8, },
}

service IdentityService {
    [post("/auth/login")]
    login(request: LoginRequest) -> LoginResult,

    [get("/users/@{user_id}")]
    get_user_by_id(user_id: uuid) -> UserProfile,

    [get("/session/identities")]
    [authorize]
    get_my_identities() -> [UserProfile],
}`;

export const SAMPLE_VOS = SAMPLE_PERSISTENCE;

export type SampleKind = "persistence" | "service";

export const SAMPLES: Record<SampleKind, string> = {
    persistence: SAMPLE_PERSISTENCE,
    service: SAMPLE_SERVICE,
};
