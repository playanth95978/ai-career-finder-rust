import { HttpResponse } from '@angular/common/http';
import { ChangeDetectionStrategy, Component, OnInit, inject, signal } from '@angular/core';
import { ReactiveFormsModule } from '@angular/forms';
import { ActivatedRoute } from '@angular/router';

import { FontAwesomeModule } from '@fortawesome/angular-fontawesome';
import { TranslatePipe } from '@ngx-translate/core';
import { Observable, finalize, map } from 'rxjs';

import { DataUtils, FileLoadError } from 'app/core/util/data-util.service';
import { EventManager, EventWithContent } from 'app/core/util/event-manager.service';
import { IJobOffer } from 'app/entities/job-offer/job-offer.model';
import { JobOfferService } from 'app/entities/job-offer/service/job-offer.service';
import { AlertError } from 'app/shared/alert/alert-error';
import { AlertErrorModel } from 'app/shared/alert/alert-error.model';
import { TranslateDirective } from 'app/shared/language';
import { IRadarHit } from '../radar-hit.model';
import { RadarHitService } from '../service/radar-hit.service';

import { RadarHitFormGroup, RadarHitFormService } from './radar-hit-form.service';

@Component({
  changeDetection: ChangeDetectionStrategy.OnPush,
  selector: 'jhi-radar-hit-update',
  templateUrl: './radar-hit-update.html',
  imports: [TranslateDirective, TranslatePipe, FontAwesomeModule, AlertError, ReactiveFormsModule],
})
export class RadarHitUpdate implements OnInit {
  readonly isSaving = signal(false);
  radarHit: IRadarHit | null = null;

  jobOffersSharedCollection = signal<IJobOffer[]>([]);

  protected dataUtils = inject(DataUtils);
  protected eventManager = inject(EventManager);
  protected radarHitService = inject(RadarHitService);
  protected radarHitFormService = inject(RadarHitFormService);
  protected jobOfferService = inject(JobOfferService);
  protected activatedRoute = inject(ActivatedRoute);

  // eslint-disable-next-line @typescript-eslint/member-ordering
  editForm: RadarHitFormGroup = this.radarHitFormService.createRadarHitFormGroup();

  compareJobOffer = (o1: IJobOffer | null, o2: IJobOffer | null): boolean => this.jobOfferService.compareJobOffer(o1, o2);

  ngOnInit(): void {
    this.activatedRoute.data.subscribe(({ radarHit }) => {
      this.radarHit = radarHit;
      if (radarHit) {
        this.updateForm(radarHit);
      }

      this.loadRelationshipsOptions();
    });
  }

  byteSize(base64String: string): string {
    return this.dataUtils.byteSize(base64String);
  }

  openFile(base64String: string, contentType: string | null | undefined): void {
    this.dataUtils.openFile(base64String, contentType);
  }

  setFileData(event: Event, field: string, isImage: boolean): void {
    this.dataUtils.loadFileToForm(event, this.editForm, field, isImage).subscribe({
      error: (err: FileLoadError) =>
        this.eventManager.broadcast(
          new EventWithContent<AlertErrorModel>('jobSearchRustApp.error', { ...err, key: `error.file.${err.key}` }),
        ),
    });
  }

  previousState(): void {
    globalThis.history.back();
  }

  save(): void {
    this.isSaving.set(true);
    const radarHit = this.radarHitFormService.getRadarHit(this.editForm);
    if (radarHit.id === null) {
      this.subscribeToSaveResponse(this.radarHitService.create(radarHit));
    } else {
      this.subscribeToSaveResponse(this.radarHitService.update(radarHit));
    }
  }

  protected subscribeToSaveResponse(result: Observable<IRadarHit | null>): void {
    result.pipe(finalize(() => this.onSaveFinalize())).subscribe({
      next: () => this.onSaveSuccess(),
      error: () => this.onSaveError(),
    });
  }

  protected onSaveSuccess(): void {
    this.previousState();
  }

  protected onSaveError(): void {
    // Api for inheritance.
  }

  protected onSaveFinalize(): void {
    this.isSaving.set(false);
  }

  protected updateForm(radarHit: IRadarHit): void {
    this.radarHit = radarHit;
    this.radarHitFormService.resetForm(this.editForm, radarHit);

    this.jobOffersSharedCollection.update(jobOffers =>
      this.jobOfferService.addJobOfferToCollectionIfMissing<IJobOffer>(jobOffers, radarHit.jobOffer),
    );
  }

  protected loadRelationshipsOptions(): void {
    this.jobOfferService
      .query()
      .pipe(map((res: HttpResponse<IJobOffer[]>) => res.body ?? []))
      .pipe(
        map((jobOffers: IJobOffer[]) =>
          this.jobOfferService.addJobOfferToCollectionIfMissing<IJobOffer>(jobOffers, this.radarHit?.jobOffer),
        ),
      )
      .subscribe((jobOffers: IJobOffer[]) => this.jobOffersSharedCollection.set(jobOffers));
  }
}
