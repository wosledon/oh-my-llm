import { useEffect } from 'react';
import {
  Title1,
  Card,
  Button,
  Badge,
  makeStyles,
  tokens,
  shorthands,
} from '@fluentui/react-components';
import { Play24Regular, Stop24Regular } from '@fluentui/react-icons';
import { useProxyStore } from '../stores/proxyStore';
import { useProviderStore } from '../stores/providerStore';
import { useModelStore } from '../stores/modelStore';
import { useI18n } from '../i18n';

const useStyles = makeStyles({
  headerRow: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    marginBottom: '24px',
  },
  grid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))',
    gap: '20px',
    marginTop: '16px',
  },
  card: {
    minHeight: '140px',
    ...shorthands.borderRadius(tokens.borderRadiusXLarge),
    backgroundColor: tokens.colorNeutralBackground1,
    ...shorthands.border('1px', 'solid', tokens.colorNeutralStroke2),
    ...shorthands.padding('20px'),
  },
  cardHeader: {
    display: 'flex',
    justifyContent: 'space-between',
    alignItems: 'flex-start',
  },
  cardTitle: {
    fontSize: tokens.fontSizeBase300,
    fontWeight: tokens.fontWeightSemibold,
    color: tokens.colorNeutralForeground2,
  },
  cardValue: {
    fontSize: tokens.fontSizeHero700,
    fontWeight: tokens.fontWeightBold,
    color: tokens.colorNeutralForeground1,
    marginTop: '12px',
  },
  cardSub: {
    fontSize: tokens.fontSizeBase200,
    color: tokens.colorNeutralForeground3,
    marginTop: '4px',
  },
  statusRunning: {
    color: tokens.colorPaletteGreenForeground1,
  },
  statusStopped: {
    color: tokens.colorPaletteRedForeground1,
  },
  statusDot: {
    display: 'inline-block',
    width: '8px',
    height: '8px',
    borderRadius: '50%',
    marginRight: '8px',
  },
  actionRow: {
    marginTop: '16px',
  },
  section: {
    marginTop: '32px',
  },
  sectionTitle: {
    marginBottom: '16px',
  },
});

export default function Dashboard() {
  const styles = useStyles();
  const { t } = useI18n();
  const { config, running, getStatus, startProxy, stopProxy } = useProxyStore();
  const { providers, fetchProviders } = useProviderStore();
  const { models, fetchModels } = useModelStore();

  useEffect(() => {
    getStatus();
    fetchProviders();
    fetchModels();
  }, [getStatus, fetchProviders, fetchModels]);

  return (
    <div>
      <div className={styles.headerRow}>
        <Title1>{t.dashboard.title}</Title1>
      </div>

      <div className={styles.grid}>
        <Card className={styles.card}>
          <div className={styles.cardHeader}>
            <span className={styles.cardTitle}>{t.dashboard.proxyStatus}</span>
            <Badge
              appearance="filled"
              color={running ? 'success' : 'danger'}
              icon={
                <span
                  className={styles.statusDot}
                  style={{ backgroundColor: running ? tokens.colorPaletteGreenBackground3 : tokens.colorPaletteRedBackground3 }}
                />
              }
            >
              {running ? t.dashboard.running : t.dashboard.stopped}
            </Badge>
          </div>
          <div className={styles.cardValue}>
            {config ? `${config.port}` : '--'}
          </div>
          <div className={styles.cardSub}>{t.dashboard.port}</div>
          <div className={styles.actionRow}>
            <Button
              icon={running ? <Stop24Regular /> : <Play24Regular />}
              appearance={running ? 'secondary' : 'primary'}
              onClick={running ? stopProxy : startProxy}
            >
              {running ? t.dashboard.stop : t.dashboard.start}
            </Button>
          </div>
        </Card>

        <Card className={styles.card}>
          <div className={styles.cardHeader}>
            <span className={styles.cardTitle}>{t.dashboard.providers}</span>
            <Badge appearance="ghost">{providers.length}</Badge>
          </div>
          <div className={styles.cardValue}>{providers.length}</div>
          <div className={styles.cardSub}>{t.dashboard.total}</div>
        </Card>

        <Card className={styles.card}>
          <div className={styles.cardHeader}>
            <span className={styles.cardTitle}>{t.dashboard.models}</span>
            <Badge appearance="ghost">{models.length}</Badge>
          </div>
          <div className={styles.cardValue}>{models.length}</div>
          <div className={styles.cardSub}>{t.dashboard.total}</div>
        </Card>
      </div>
    </div>
  );
}
