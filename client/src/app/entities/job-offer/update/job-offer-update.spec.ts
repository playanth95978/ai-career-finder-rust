import { beforeEach, describe, expect, it, vitest } from 'vitest';
import { provideHttpClientTesting } from '@angular/common/http/testing';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { ActivatedRoute } from '@angular/router';

import { provideTranslateService } from '@ngx-translate/core';
import { Subject, from, of } from 'rxjs';

import { IJobOffer } from '../job-offer.model';
import { JobOfferService } from '../service/job-offer.service';

import { JobOfferFormService } from './job-offer-form.service';
import { JobOfferUpdate } from './job-offer-update';

describe('JobOffer Management Update Component', () => {
  let comp: JobOfferUpdate;
  let fixture: ComponentFixture<JobOfferUpdate>;
  let activatedRoute: ActivatedRoute;
  let jobOfferFormService: JobOfferFormService;
  let jobOfferService: JobOfferService;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [
        provideTranslateService(),
        provideHttpClientTesting(),
        {
          provide: ActivatedRoute,
          useValue: {
            params: from([{}]),
          },
        },
      ],
    });

    fixture = TestBed.createComponent(JobOfferUpdate);
    activatedRoute = TestBed.inject(ActivatedRoute);
    jobOfferFormService = TestBed.inject(JobOfferFormService);
    jobOfferService = TestBed.inject(JobOfferService);

    comp = fixture.componentInstance;
  });

  describe('ngOnInit', () => {
    it('should update editForm', () => {
      const jobOffer: IJobOffer = { id: 5985 };

      activatedRoute.data = of({ jobOffer });
      comp.ngOnInit();

      expect(comp.jobOffer).toEqual(jobOffer);
    });
  });

  describe('save', () => {
    it('should call update service on save for existing entity', () => {
      // GIVEN
      const saveSubject = new Subject<IJobOffer>();
      const jobOffer = { id: 9246 };
      vitest.spyOn(jobOfferFormService, 'getJobOffer').mockReturnValue(jobOffer);
      vitest.spyOn(jobOfferService, 'update').mockReturnValue(saveSubject);
      vitest.spyOn(comp, 'previousState');
      activatedRoute.data = of({ jobOffer });
      comp.ngOnInit();

      // WHEN
      comp.save();
      expect(comp.isSaving()).toEqual(true);
      saveSubject.next(jobOffer);
      saveSubject.complete();

      // THEN
      expect(jobOfferFormService.getJobOffer).toHaveBeenCalled();
      expect(comp.previousState).toHaveBeenCalled();
      expect(jobOfferService.update).toHaveBeenCalledWith(expect.objectContaining(jobOffer));
      expect(comp.isSaving()).toEqual(false);
    });

    it('should call create service on save for new entity', () => {
      // GIVEN
      const saveSubject = new Subject<IJobOffer>();
      const jobOffer = { id: 9246 };
      vitest.spyOn(jobOfferFormService, 'getJobOffer').mockReturnValue({ id: null });
      vitest.spyOn(jobOfferService, 'create').mockReturnValue(saveSubject);
      vitest.spyOn(comp, 'previousState');
      activatedRoute.data = of({ jobOffer: null });
      comp.ngOnInit();

      // WHEN
      comp.save();
      expect(comp.isSaving()).toEqual(true);
      saveSubject.next(jobOffer);
      saveSubject.complete();

      // THEN
      expect(jobOfferFormService.getJobOffer).toHaveBeenCalled();
      expect(jobOfferService.create).toHaveBeenCalled();
      expect(comp.isSaving()).toEqual(false);
      expect(comp.previousState).toHaveBeenCalled();
    });

    it('should set isSaving to false on error', () => {
      // GIVEN
      const saveSubject = new Subject<IJobOffer>();
      const jobOffer = { id: 9246 };
      vitest.spyOn(jobOfferService, 'update').mockReturnValue(saveSubject);
      vitest.spyOn(comp, 'previousState');
      activatedRoute.data = of({ jobOffer });
      comp.ngOnInit();

      // WHEN
      comp.save();
      expect(comp.isSaving()).toEqual(true);
      saveSubject.error('This is an error!');

      // THEN
      expect(jobOfferService.update).toHaveBeenCalled();
      expect(comp.isSaving()).toEqual(false);
      expect(comp.previousState).not.toHaveBeenCalled();
    });
  });
});
