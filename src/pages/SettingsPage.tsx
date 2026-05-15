import { useEffect } from 'react';
import {
  Title1,
  Title2,
  Label,
  Input,
  Switch,
  Dropdown,
  Option,
  makeStyles,
  Spinner,
  Card,
  shorthands,
  tokens,
  Badge,
} from '@fluentui/react-components';
import { useProxyStore } from '../stores/proxyStore';
import { useI18n } from '../i18n';

const useStyles = makeStyles({
  section: {
    marginTop: '24px',
  },
  sectionTitle: {
    marginBottom: '16px',
  },
  card: {
    backgroundColor: tokens.colorNeutralBackground1,
    ...shorthands.border('1px', 'solid', tokens.colorNeutralStroke2),
    ...shorthands.borderRadius(tokens.borderRadiusXLarge),
    ...shorthands.padding('24px'),
    maxWidth: '560px',
  },
  field: {
    display: 'flex',
    flexDirection: 'column',
    gap: '6px',
    marginBottom: '20px',
  },
  fieldRow: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    marginBottom: '20px',
  },
  labelGroup: {
    display: 'flex',
    flexDirection: 'column',
    gap: '2px',
  },
  fieldLabel: {
    fontSize: tokens.fontSizeBase300,
    fontWeight: tokens.fontWeightSemibold,
    color: tokens.colorNeutralForeground1,
  },
  fieldDesc: {
    fontSize: tokens.fontSizeBase200,
    color: tokens.colorNeutralForeground3,
  },
});

export default function SettingsPage() {
  const styles = useStyles();
  const { t, lang, setLang } = useI18n();
  const { config, loading, error, fetchConfig, updateConfig } = useProxyStore();

  useEffect(() => {
    fetchConfig();
  }, [fetchConfig]);

  if (!config) return <Spinner label={t.common.loading} />;

  return (
    <div>
      <Title1>{t.settings.title}</Title1>

      {loading && <Spinner label={t.common.loading} />}
      {error && (
        <Badge appearance="filled" color="danger" style={{ marginTop: '12px', display: 'block' }}>
          {t.common.error}: {error}
        </Badge>
      )}

      <div className={styles.section}>
        <Title2 className={styles.sectionTitle}>{t.settings.title}</Title2>
        <Card className={styles.card}>
          <div className={styles.field}>
            <Label className={styles.fieldLabel}>{t.settings.port}</Label>
            <Input
              type="number"
              value={String(config.port)}
              onChange={(e: React.ChangeEvent<HTMLInputElement>) => updateConfig({ ...config, port: parseInt(e.target.value) || 11888 })}
            />
          </div>

          <div className={styles.fieldRow}>
            <div className={styles.labelGroup}>
              <span className={styles.fieldLabel}>{t.settings.openaiEnabled}</span>
            </div>
            <Switch
              checked={config.openai_enabled}
              onChange={(e: React.ChangeEvent<HTMLInputElement>) => updateConfig({ ...config, openai_enabled: e.target.checked })}
            />
          </div>

          <div className={styles.fieldRow}>
            <div className={styles.labelGroup}>
              <span className={styles.fieldLabel}>{t.settings.anthropicEnabled}</span>
            </div>
            <Switch
              checked={config.anthropic_enabled}
              onChange={(e: React.ChangeEvent<HTMLInputElement>) => updateConfig({ ...config, anthropic_enabled: e.target.checked })}
            />
          </div>

          <div className={styles.fieldRow}>
            <div className={styles.labelGroup}>
              <span className={styles.fieldLabel}>{t.settings.autoStart}</span>
            </div>
            <Switch
              checked={config.auto_start}
              onChange={(e: React.ChangeEvent<HTMLInputElement>) => updateConfig({ ...config, auto_start: e.target.checked })}
            />
          </div>

          <div className={styles.fieldRow}>
            <div className={styles.labelGroup}>
              <span className={styles.fieldLabel}>{t.settings.logRequests}</span>
            </div>
            <Switch
              checked={config.log_requests}
              onChange={(e: React.ChangeEvent<HTMLInputElement>) => updateConfig({ ...config, log_requests: e.target.checked })}
            />
          </div>

          <div className={styles.field}>
            <Label className={styles.fieldLabel}>{t.settings.logRetentionDays}</Label>
            <Input
              type="number"
              value={String(config.log_retention_days)}
              onChange={(e: React.ChangeEvent<HTMLInputElement>) => updateConfig({ ...config, log_retention_days: parseInt(e.target.value) || 30 })}
            />
          </div>

          <div className={styles.field}>
            <Label className={styles.fieldLabel}>{t.settings.maxRetries}</Label>
            <Input
              type="number"
              value={String(config.max_retries)}
              onChange={(e: React.ChangeEvent<HTMLInputElement>) => updateConfig({ ...config, max_retries: parseInt(e.target.value) || 3 })}
            />
          </div>

          <div className={styles.field}>
            <Label className={styles.fieldLabel}>{t.settings.timeoutSecs}</Label>
            <Input
              type="number"
              value={String(config.timeout_secs)}
              onChange={(e: React.ChangeEvent<HTMLInputElement>) => updateConfig({ ...config, timeout_secs: parseInt(e.target.value) || 120 })}
            />
          </div>
        </Card>
      </div>

      <div className={styles.section}>
        <Title2 className={styles.sectionTitle}>{t.settings.language}</Title2>
        <Card className={styles.card}>
          <div className={styles.field}>
            <Label className={styles.fieldLabel}>{t.settings.language}</Label>
            <Dropdown value={lang} onOptionSelect={(_: unknown, data: { optionValue?: string }) => setLang((data.optionValue as 'zh' | 'en') || 'zh')}>
              <Option value="zh">{t.settings.zh}</Option>
              <Option value="en">{t.settings.en}</Option>
            </Dropdown>
          </div>
        </Card>
      </div>
    </div>
  );
}
