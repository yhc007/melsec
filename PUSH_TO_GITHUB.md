# GitHub 푸시 가이드

## 현재 상태

✅ Git 저장소 초기화 완료
✅ 모든 파일 커밋 완료 (17개 파일)
✅ Remote 저장소 설정 완료 (https://github.com/yhc007/melsec.git)
✅ 커밋 메시지: "Initial commit: MELSEC PLC 통신 라이브러리 및 TUI 모니터링 프로그램"

## GitHub 푸시 방법

### 방법 1: Personal Access Token 사용 (권장)

1. GitHub에서 Personal Access Token 생성:
   - GitHub 웹사이트 접속
   - Settings > Developer settings > Personal access tokens > Tokens (classic)
   - Generate new token (classic)
   - `repo` 권한 선택
   - 토큰 생성 후 복사

2. 푸시 실행:
```bash
# 방법 A: URL에 토큰 포함
git remote set-url origin https://YOUR_TOKEN@github.com/yhc007/melsec.git
git push -u origin main

# 방법 B: 명령어 실행 시 입력
git push -u origin main
# Username: yhc007
# Password: YOUR_TOKEN (토큰 입력)
```

### 방법 2: SSH 사용

1. SSH 키 생성 (이미 있다면 생략):
```bash
ssh-keygen -t ed25519 -C "yhc007@users.noreply.github.com"
cat ~/.ssh/id_ed25519.pub
# 출력된 키를 GitHub에 추가 (Settings > SSH and GPG keys)
```

2. Remote URL 변경 및 푸시:
```bash
git remote set-url origin git@github.com:yhc007/melsec.git
git push -u origin main
```

### 방법 3: GitHub CLI 사용

```bash
# GitHub CLI 설치 (Ubuntu/Debian)
sudo apt-get install gh

# 로그인
gh auth login

# 푸시
git push -u origin main
```

## 포함된 파일

- 소스 코드:
  - `src/` 디렉토리 (모든 Rust 소스 파일)
  - `Cargo.toml` (프로젝트 설정)
  - `.gitignore` (Git 무시 파일)

- 문서:
  - `README.md` (프로젝트 설명)
  - `RUN.md` (GUI 버전 실행 가이드)
  - `RUN_TUI.md` (TUI 버전 실행 가이드)
  - `TROUBLESHOOTING.md` (문제 해결 가이드)

- 스크립트:
  - `run_gui.sh` (GUI 실행 스크립트)
  - `quick_start.sh` (빠른 시작 스크립트)

- 기타:
  - `PROBLEMS.md`, `ISSUES_FOUND.md` (개발 과정 문서)

## 다음 단계

위 방법 중 하나를 선택하여 GitHub에 푸시하세요.

푸시 후 확인:
```bash
git remote -v
git log --oneline
```

